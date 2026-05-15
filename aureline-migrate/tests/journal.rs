mod common;

use aureline_migrate::journal::{
    Journal, JournalEntry, append_entry, next_idx, read_journal, validate_slug, write_journal,
};
use chrono::Utc;

use common::temp_dir;

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
