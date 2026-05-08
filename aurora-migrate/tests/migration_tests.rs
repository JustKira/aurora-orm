use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use aurora_core::ast::{Field, Schema, SchemaItem, Table, Type};
use aurora_migrate::checksum::sha256_hex;
use aurora_migrate::config::{Config, MigrationsConfig, SchemaConfig};
use aurora_migrate::diff::diff_schemas;
use aurora_migrate::error::Error;
use aurora_migrate::fs_io::{read_previous_schema, read_schema};
use aurora_migrate::journal::{
    Journal, JournalEntry, append_entry, next_idx, read_journal, validate_slug, write_journal,
};
use aurora_migrate::ops::Op;
use aurora_migrate::render::{emit_down, emit_up};
use aurora_migrate::snapshot::{canonicalize, parse_snapshot};
use aurora_migrate::{GenerateOpts, generate};
use chrono::Utc;

#[test]
fn diffs_stably_and_marks_destructive_ops() {
    let prev = schema(vec![table(
        "User",
        Some("schemafull"),
        vec![
            field("age", "int", true),
            field("email", "string", false),
            field("name", "string", false),
        ],
    )]);
    let next = schema(vec![table(
        "User",
        Some("schemaless"),
        vec![
            field("age", "int", false),
            field("email", "datetime", false),
            field("score", "float", true),
        ],
    )]);

    let ops = diff_schemas(&prev, &next);
    assert_eq!(ops.len(), 5);
    assert!(matches!(ops[0], Op::ChangeTableMode { .. }));
    assert!(matches!(
        ops[1],
        Op::ChangeFieldOptional {
            now_optional: false,
            ..
        }
    ));
    assert!(matches!(ops[2], Op::ChangeFieldType { .. }));
    assert!(matches!(ops[3], Op::RemoveField { .. }));
    assert!(matches!(ops[4], Op::AddField { .. }));
    assert_eq!(ops.iter().filter(|op| op.destructive()).count(), 3);
}

#[test]
fn snapshots_are_canonical_and_roundtrip() {
    let a = Schema {
        items: vec![
            SchemaItem::DocComment {
                text: "ignored".to_string(),
            },
            SchemaItem::TableDecl(table(
                "User",
                None,
                vec![field("b", "int", false), field("a", "string", true)],
            )),
        ],
    };
    let b = schema(vec![table(
        "User",
        None,
        vec![field("a", "string", true), field("b", "int", false)],
    )]);

    let canonical = canonicalize(&a);
    assert_eq!(canonical, canonicalize(&b));
    assert_eq!(parse_snapshot(&canonical).unwrap(), b);
    assert!(matches!(
        parse_snapshot(r#"{"version":2,"tables":[]}"#),
        Err(Error::SnapshotDecode(_))
    ));
}

#[test]
fn render_up_and_down() {
    let ops = vec![
        Op::CreateTable(table(
            "User",
            Some("schemafull"),
            vec![field("email", "string", false)],
        )),
        Op::AddField {
            table: "User".to_string(),
            field: field("score", "float", true),
        },
        Op::RemoveField {
            table: "User".to_string(),
            field: "legacy".to_string(),
        },
    ];

    assert_eq!(
        emit_up(&ops),
        "DEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD email ON user TYPE string;\nDEFINE FIELD score ON user TYPE option<float>;\nREMOVE FIELD legacy ON TABLE user;\n"
    );
    assert_eq!(
        emit_down(&ops),
        "-- down: RemoveField User.legacy cannot restore data\nREMOVE FIELD score ON TABLE user;\nREMOVE TABLE user;\n"
    );
}

#[test]
fn combined_type_and_optional_change_stays_one_op_but_renders_new_optionality() {
    let prev = schema(vec![table("User", None, vec![field("age", "int", false)])]);
    let next = schema(vec![table(
        "User",
        None,
        vec![field("age", "string", true)],
    )]);

    let ops = diff_schemas(&prev, &next);
    assert_eq!(ops.len(), 1);
    assert!(matches!(ops[0], Op::ChangeFieldType { .. }));
    assert_eq!(
        emit_up(&ops),
        "ALTER FIELD age ON user TYPE option<string>;\n"
    );
}

#[test]
fn drop_modifier_is_a_table_mode_not_table_removal() {
    let prev = schema(vec![table("Review", None, vec![])]);
    let next = schema(vec![table(
        "Review",
        Some("drop"),
        vec![field("rating", "float", false)],
    )]);

    let ops = diff_schemas(&prev, &next);
    assert!(matches!(
        &ops[..],
        [Op::ChangeTableMode {
            table,
            from: None,
            to: Some(to),
        }, Op::AddField { table: field_table, .. }]
            if table == "Review" && to == "drop" && field_table == "Review"
    ));
    assert_eq!(
        emit_up(&[Op::CreateTable(table(
            "Review",
            Some("drop"),
            vec![field("rating", "float", false)]
        ))]),
        "DEFINE TABLE review DROP;\nDEFINE FIELD rating ON review TYPE float;\n"
    );
}

#[test]
fn journal_helpers_validate_and_roundtrip() {
    let dir = temp_dir("journal");
    let meta = dir.join("meta");
    assert_eq!(read_journal(&meta).unwrap().entries, Vec::new());
    assert!(validate_slug("init").is_ok());
    assert!(validate_slug("add_users").is_ok());
    assert!(validate_slug("0000_init").is_err());
    assert!(validate_slug("Bad").is_err());
    assert_eq!(
        next_idx(&Journal {
            version: 1,
            entries: vec![]
        }),
        0
    );

    let entry = JournalEntry {
        idx: 0,
        name: "init".to_string(),
        created_at: Utc::now(),
        checksum: "abc".to_string(),
        destructive: false,
        warnings: vec![],
        kind: "generated".to_string(),
    };
    write_journal(
        &meta,
        &Journal {
            version: 1,
            entries: vec![entry.clone()],
        },
    )
    .unwrap();
    assert_eq!(read_journal(&meta).unwrap().entries, vec![entry.clone()]);
    assert!(append_entry(&meta, entry).is_err());
}

#[test]
fn generate_creates_files_and_uses_previous_snapshot() {
    let dir = temp_dir("generate");
    let schema_path = dir.join("schema.aurora");
    let migrations_dir = dir.join("migrations");
    fs::write(&schema_path, "table User schemafull { email string }\n").unwrap();

    let report = generate(GenerateOpts {
        config: config(schema_path.clone(), migrations_dir.clone()),
        name: "init".to_string(),
        allow_empty: false,
    })
    .unwrap();

    assert_eq!(report.idx, 0);
    let migration = fs::read_to_string(migrations_dir.join("0000_init/migration.surql")).unwrap();
    assert_eq!(
        migration,
        "DEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD email ON user TYPE string;\n"
    );
    let journal = read_journal(&migrations_dir.join("meta")).unwrap();
    assert_eq!(
        journal.entries[0].checksum,
        sha256_hex(migration.as_bytes())
    );
    assert!(migrations_dir.join("migration_lock.toml").exists());

    assert!(matches!(
        generate(GenerateOpts {
            config: config(schema_path.clone(), migrations_dir.clone()),
            name: "noop".to_string(),
            allow_empty: false,
        }),
        Err(Error::EmptyDiff)
    ));

    fs::write(
        &schema_path,
        "table User schemafull { email string score float? }\n",
    )
    .unwrap();
    let second = generate(GenerateOpts {
        config: config(schema_path, migrations_dir.clone()),
        name: "add_score".to_string(),
        allow_empty: false,
    })
    .unwrap();
    assert_eq!(second.idx, 1);
    assert_eq!(second.ops[0].summary(), "+ ADD FIELD User.score");
}

#[test]
fn generate_accepts_matching_lockfile() {
    let dir = temp_dir("lock_ok");
    let schema_path = dir.join("schema.aurora");
    let migrations_dir = dir.join("migrations");
    fs::create_dir_all(&migrations_dir).unwrap();
    fs::write(&schema_path, "table User { email string }\n").unwrap();
    fs::write(
        migrations_dir.join("migration_lock.toml"),
        "provider = \"surrealdb\"\nsnapshot_version = 1\n",
    )
    .unwrap();

    generate(GenerateOpts {
        config: config(schema_path, migrations_dir),
        name: "init".to_string(),
        allow_empty: false,
    })
    .unwrap();
}

#[test]
fn generate_rejects_incompatible_lockfile() {
    let dir = temp_dir("lock_bad_provider");
    let schema_path = dir.join("schema.aurora");
    let migrations_dir = dir.join("migrations");
    fs::create_dir_all(&migrations_dir).unwrap();
    fs::write(&schema_path, "table User { email string }\n").unwrap();
    fs::write(
        migrations_dir.join("migration_lock.toml"),
        "provider = \"postgres\"\nsnapshot_version = 1\n",
    )
    .unwrap();

    let result = generate(GenerateOpts {
        config: config(schema_path, migrations_dir),
        name: "init".to_string(),
        allow_empty: false,
    });
    assert!(matches!(result, Err(Error::Lock { .. })));
}

#[test]
fn generate_rejects_unsupported_lockfile_snapshot_version() {
    let dir = temp_dir("lock_bad_snapshot");
    let schema_path = dir.join("schema.aurora");
    let migrations_dir = dir.join("migrations");
    fs::create_dir_all(&migrations_dir).unwrap();
    fs::write(&schema_path, "table User { email string }\n").unwrap();
    fs::write(
        migrations_dir.join("migration_lock.toml"),
        "provider = \"surrealdb\"\nsnapshot_version = 2\n",
    )
    .unwrap();

    let result = generate(GenerateOpts {
        config: config(schema_path, migrations_dir),
        name: "init".to_string(),
        allow_empty: false,
    });
    assert!(matches!(result, Err(Error::Lock { .. })));
}

#[test]
fn generate_rejects_malformed_lockfile() {
    let dir = temp_dir("lock_malformed");
    let schema_path = dir.join("schema.aurora");
    let migrations_dir = dir.join("migrations");
    fs::create_dir_all(&migrations_dir).unwrap();
    fs::write(&schema_path, "table User { email string }\n").unwrap();
    fs::write(migrations_dir.join("migration_lock.toml"), "not = [valid").unwrap();

    let result = generate(GenerateOpts {
        config: config(schema_path, migrations_dir),
        name: "init".to_string(),
        allow_empty: false,
    });
    assert!(matches!(result, Err(Error::Lock { .. })));
}

#[test]
fn fs_io_reads_schema_and_latest_snapshot() {
    let dir = temp_dir("fs_io");
    let schema_path = dir.join("schema.aurora");
    let meta_dir = dir.join("migrations/meta");
    fs::create_dir_all(&meta_dir).unwrap();
    fs::write(&schema_path, "table User { email string }\n").unwrap();
    let parsed = read_schema(&schema_path).unwrap();
    fs::write(meta_dir.join("0000_snapshot.json"), canonicalize(&parsed)).unwrap();
    let journal = Journal {
        version: 1,
        entries: vec![JournalEntry {
            idx: 0,
            name: "init".to_string(),
            created_at: Utc::now(),
            checksum: "abc".to_string(),
            destructive: false,
            warnings: vec![],
            kind: "generated".to_string(),
        }],
    };

    assert_eq!(read_previous_schema(&meta_dir, &journal).unwrap(), parsed);
}

fn schema(tables: Vec<Table>) -> Schema {
    Schema {
        items: tables.into_iter().map(SchemaItem::TableDecl).collect(),
    }
}

fn table(name: &str, modifier: Option<&str>, fields: Vec<Field>) -> Table {
    Table {
        name: name.to_string(),
        modifier: modifier.map(str::to_string),
        fields,
        indexes: Vec::new(),
        raw_attributes: Vec::new(),
    }
}

fn field(name: &str, type_name: &str, optional: bool) -> Field {
    Field {
        name: name.to_string(),
        ty: Type::primitive(type_name),
        optional,
        flexible: false,
        raw_attributes: Vec::new(),
    }
}

fn config(schema: PathBuf, migrations: PathBuf) -> Config {
    Config {
        schema: SchemaConfig { file: schema },
        migrations: MigrationsConfig { dir: migrations },
        database: Default::default(),
    }
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("aurora_migrate_{label}_{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}
