#[macro_use]
mod common;

use std::fs;

use aureline_migrate::checksum::sha256_hex;
use aureline_migrate::error::Error;
use aureline_migrate::journal::read_journal;
use aureline_migrate::snapshot::parse_snapshot;
use aureline_migrate::{GenerateOpts, generate};

use common::{config, temp_dir};

fn write_schema(path: &std::path::Path, schema: &str) {
    fs::write(path, schema).unwrap();
}

fn generate_named(
    schema_path: std::path::PathBuf,
    migrations_dir: std::path::PathBuf,
    name: &str,
) -> aureline_migrate::GenerateReport {
    generate(GenerateOpts {
        config: config(schema_path, migrations_dir),
        name: name.to_string(),
        allow_empty: false,
    })
    .unwrap()
}

fn migration_sql(migrations_dir: &std::path::Path, dir: &str) -> String {
    fs::read_to_string(migrations_dir.join(dir).join("migration.surql")).unwrap()
}

#[test]
fn generate_creates_files_and_uses_previous_snapshot() {
    let dir = temp_dir("generate");
    let schema_path = dir.join("schema.aureline");
    let migrations_dir = dir.join("migrations");
    fs::write(&schema_path, "table User schemafull {\n  email string\n}\n").unwrap();

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
        expected_surql!(
            "DEFINE TABLE user SCHEMAFULL;",
            "DEFINE FIELD email ON user TYPE string;",
        )
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
        "table User schemafull {\n  email string\n  score float?\n}\n",
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
fn generate_includes_standard_unique_and_count_indexes() {
    let dir = temp_dir("generate_indexes");
    let schema_path = dir.join("schema.aureline");
    let migrations_dir = dir.join("migrations");
    write_schema(
        &schema_path,
        aureline_schema!(
            "table User schemafull {",
            "  account string",
            "  email string @unique",
            "  status string @index(name: idx_user_status)",
            "",
            "  @@index(fields: [account, status], name: account_status_lookup)",
            "  @@unique(fields: [account, email])",
            "  @@count",
            "}",
        ),
    );

    let report = generate_named(schema_path, migrations_dir.clone(), "init_indexes");
    let migration = migration_sql(&migrations_dir, "0000_init_indexes");

    assert_eq!(report.idx, 0);
    assert_eq!(
        migration,
        expected_surql!(
            "DEFINE TABLE user SCHEMAFULL;",
            "DEFINE FIELD account ON user TYPE string;",
            "DEFINE FIELD email ON user TYPE string;",
            "DEFINE FIELD status ON user TYPE string;",
            "DEFINE INDEX account_status_lookup ON user FIELDS account, status;",
            "DEFINE INDEX idx_user_status ON user FIELDS status;",
            "DEFINE INDEX user_account_email_unique ON user FIELDS account, email UNIQUE;",
            "DEFINE INDEX user_count ON user COUNT;",
            "DEFINE INDEX user_email_unique ON user FIELDS email UNIQUE;",
        )
    );

    let snapshot = fs::read_to_string(migrations_dir.join("meta/0000_snapshot.json")).unwrap();
    let parsed = parse_snapshot(&snapshot).unwrap();
    assert!(format!("{parsed:?}").contains("idx_user_status"));
    assert!(format!("{parsed:?}").contains("user_count"));
}

#[test]
fn generate_includes_fulltext_analyzer_and_hnsw_indexes() {
    let dir = temp_dir("generate_search_indexes");
    let schema_path = dir.join("schema.aureline");
    let migrations_dir = dir.join("migrations");
    write_schema(
        &schema_path,
        aureline_schema!(
            "analyzer edu_analyzer {",
            "  tokenizers blank, class",
            "  filters lowercase, snowball(english)",
            "}",
            "",
            "table Document schemafull {",
            "  body string @fulltext(analyzer: edu_analyzer, bm25: (1.2, 0.75), highlights: true)",
            "  title string @fulltext(analyzer: edu_analyzer)",
            "  v_minimal array<float> @hnsw(dimension: 384)",
            "  v_dist array<float> @hnsw(dimension: 768, dist: cosine)",
            "  v_tuned array<float> @hnsw(dimension: 1536, dist: euclidean, type: f32, efc: 200, m: 16)",
            "}",
        ),
    );

    generate_named(schema_path, migrations_dir.clone(), "search_indexes");
    let migration = migration_sql(&migrations_dir, "0000_search_indexes");

    assert_eq!(
        migration,
        expected_surql!(
            "DEFINE ANALYZER edu_analyzer TOKENIZERS blank,class FILTERS lowercase,snowball(english);",
            "DEFINE TABLE document SCHEMAFULL;",
            "DEFINE FIELD body ON document TYPE string;",
            "DEFINE FIELD title ON document TYPE string;",
            "DEFINE FIELD v_dist ON document TYPE array<float>;",
            "DEFINE FIELD v_minimal ON document TYPE array<float>;",
            "DEFINE FIELD v_tuned ON document TYPE array<float>;",
            "DEFINE INDEX document_body_fts ON document FIELDS body FULLTEXT ANALYZER edu_analyzer BM25(1.2, 0.75) HIGHLIGHTS;",
            "DEFINE INDEX document_title_fts ON document FIELDS title FULLTEXT ANALYZER edu_analyzer;",
            "DEFINE INDEX document_v_dist_hnsw ON document FIELDS v_dist HNSW DIMENSION 768 DIST COSINE;",
            "DEFINE INDEX document_v_minimal_hnsw ON document FIELDS v_minimal HNSW DIMENSION 384;",
            "DEFINE INDEX document_v_tuned_hnsw ON document FIELDS v_tuned HNSW DIMENSION 1536 TYPE F32 DIST EUCLIDEAN EFC 200 M 16;",
        )
    );
}

#[test]
fn generate_detects_index_changes_from_previous_snapshot() {
    let dir = temp_dir("generate_index_changes");
    let schema_path = dir.join("schema.aureline");
    let migrations_dir = dir.join("migrations");
    write_schema(
        &schema_path,
        aureline_schema!(
            "table User {",
            "  email string @index(name: old_email_lookup)",
            "}",
        ),
    );
    generate_named(schema_path.clone(), migrations_dir.clone(), "init");

    write_schema(
        &schema_path,
        aureline_schema!(
            "table User {",
            "  email string @unique(name: new_email_constraint)",
            "}",
        ),
    );
    generate_named(schema_path, migrations_dir.clone(), "change_index");
    let migration = migration_sql(&migrations_dir, "0001_change_index");

    assert_eq!(
        migration,
        expected_surql!(
            "REMOVE INDEX old_email_lookup ON TABLE user;",
            "DEFINE INDEX new_email_constraint ON user FIELDS email UNIQUE;",
        )
    );
}

#[test]
fn generate_accepts_matching_lockfile() {
    let dir = temp_dir("lock_ok");
    let schema_path = dir.join("schema.aureline");
    let migrations_dir = dir.join("migrations");
    fs::create_dir_all(&migrations_dir).unwrap();
    fs::write(&schema_path, "table User {\n  email string\n}\n").unwrap();
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
    let schema_path = dir.join("schema.aureline");
    let migrations_dir = dir.join("migrations");
    fs::create_dir_all(&migrations_dir).unwrap();
    fs::write(&schema_path, "table User {\n  email string\n}\n").unwrap();
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
    let schema_path = dir.join("schema.aureline");
    let migrations_dir = dir.join("migrations");
    fs::create_dir_all(&migrations_dir).unwrap();
    fs::write(&schema_path, "table User {\n  email string\n}\n").unwrap();
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
    let schema_path = dir.join("schema.aureline");
    let migrations_dir = dir.join("migrations");
    fs::create_dir_all(&migrations_dir).unwrap();
    fs::write(&schema_path, "table User {\n  email string\n}\n").unwrap();
    fs::write(migrations_dir.join("migration_lock.toml"), "not = [valid").unwrap();

    let result = generate(GenerateOpts {
        config: config(schema_path, migrations_dir),
        name: "init".to_string(),
        allow_empty: false,
    });
    assert!(matches!(result, Err(Error::Lock { .. })));
}
