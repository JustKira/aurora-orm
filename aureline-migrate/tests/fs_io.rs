mod common;

use std::fs;

use aureline_migrate::error::Error;
use aureline_migrate::fs_io::{read_previous_schema, read_schema};
use aureline_migrate::journal::{Journal, JournalEntry};
use aureline_migrate::snapshot::canonicalize;
use chrono::Utc;

use common::temp_dir;

#[test]
fn fs_io_reads_schema_and_latest_snapshot() {
    let dir = temp_dir("fs_io");
    let schema_path = dir.join("schema.aureline");
    let meta_dir = dir.join("migrations/meta");
    fs::create_dir_all(&meta_dir).unwrap();
    fs::write(&schema_path, "table User {\n  email string\n}\n").unwrap();
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

#[test]
fn fs_io_rejects_surql_blocks_for_migrations() {
    let dir = temp_dir("fs_io_surql");
    let schema_path = dir.join("schema.aureline");
    fs::write(&schema_path, "#surql { RETURN 1; }\n").unwrap();

    let result = read_schema(&schema_path);

    match result {
        Err(Error::UnsupportedSchemaItem { message, .. }) => {
            assert!(message.contains("#surql blocks are not yet supported"));
        }
        other => panic!("expected top-level #surql migration rejection, got {other:?}"),
    }
}
