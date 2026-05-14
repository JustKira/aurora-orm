#[macro_use]
mod common;

use std::fs;

use aureline_migrate::checksum::sha256_hex;
use aureline_migrate::error::Error;
use aureline_migrate::journal::read_journal;
use aureline_migrate::{GenerateOpts, generate};

use common::{config, temp_dir};

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
