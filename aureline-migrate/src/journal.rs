use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result, io};
use crate::fs_io::write_atomic;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Journal {
    pub version: u32,
    pub entries: Vec<JournalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JournalEntry {
    /// Numeric index that prefixes the migration folder (e.g. `0007_*` → 7).
    /// Acts as the primary key of the journal — duplicates are rejected.
    pub idx: u32,
    /// Human-readable slug that follows the index in the folder name
    /// (`0007_add_users` → `add_users`). Purely a label for humans/log output.
    pub name: String,
    /// UTC timestamp the migration was generated. Used for audit trails and
    /// "when did this start failing?" debugging — not for ordering (use `idx`).
    pub created_at: DateTime<Utc>,
    /// SHA-256 of `migration.surql` at the moment it was written. Lets later
    /// tools detect if the file has been hand-edited or corrupted since then.
    pub checksum: String,
    /// True if any op in this migration drops data (RemoveTable, RemoveField,
    /// type change, etc.). Precomputed so an `apply` step can warn the user
    /// without re-parsing the SQL.
    pub destructive: bool,
    /// Human-readable notes about what the *down* migration cannot recover
    /// (e.g. "RemoveTable cannot restore data"). Stored here so they can be
    /// displayed at apply/rollback time without re-running the diff.
    pub warnings: Vec<String>,
    /// How this entry was produced. Currently always `"generated"`; reserved
    /// for future kinds like `"manual"` or `"squashed"`.
    #[serde(default = "default_kind")]
    pub kind: String,
}

pub fn default_kind() -> String {
    "generated".to_string()
}

pub fn read_journal(meta_dir: &Path) -> Result<Journal> {
    let path = meta_dir.join("_journal.json");
    match fs::read_to_string(&path) {
        Ok(contents) => {
            let journal = serde_json::from_str::<Journal>(&contents)
                .map_err(|error| Error::Journal(error.to_string()))?;
            if journal.version != 1 {
                return Err(Error::Journal(format!(
                    "unsupported journal version {}",
                    journal.version
                )));
            }
            Ok(journal)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Journal {
            version: 1,
            entries: Vec::new(),
        }),
        Err(error) => Err(io(path, error)),
    }
}

/// Saves the journal, replacing the file on disk. A direct write risks
/// truncating the file (and losing all migration history) if it crashes
/// mid-write, so we stage to a tmp sibling and atomically rename into place.
pub fn write_journal(meta_dir: &Path, journal: &Journal) -> Result<()> {
    fs::create_dir_all(meta_dir).map_err(|error| io(meta_dir, error))?;
    let path = meta_dir.join("_journal.json");
    let contents =
        serde_json::to_string_pretty(journal).map_err(|error| Error::Journal(error.to_string()))?;
    write_atomic(&path, contents.as_bytes())
}

/// The only sanctioned way to add an entry: rejects duplicate `idx` so two
/// migrations sharing a number (e.g. parallel branches) fail loudly instead
/// of silently overwriting each other.
pub fn append_entry(meta_dir: &Path, entry: JournalEntry) -> Result<()> {
    // Future work: if Phase 1 ever runs in CI/server workflows, wrap this
    // read/mutate/write sequence in a filesystem lock.
    let mut journal = read_journal(meta_dir)?;
    if journal
        .entries
        .iter()
        .any(|existing| existing.idx == entry.idx)
    {
        return Err(Error::Journal(format!(
            "duplicate migration idx {}",
            entry.idx
        )));
    }
    journal.entries.push(entry);
    journal.entries.sort_by_key(|entry| entry.idx);
    write_journal(meta_dir, &journal)
}

/// Picks the index for the next migration as `max(idx) + 1` (not `len()`),
/// so gaps from future squash/delete operations don't recycle old numbers.
/// Conceptually the same as a SQL `AUTO_INCREMENT` primary key.
pub fn next_idx(journal: &Journal) -> u32 {
    journal
        .entries
        .iter()
        .map(|entry| entry.idx)
        .max()
        .map_or(0, |idx| idx + 1)
}

/// Validates the human-supplied slug used in a migration folder name
/// (e.g. the `add_users` in `migrations/0007_add_users/`). Restricted to a
/// portable filesystem-safe subset so paths behave the same across OSes.
pub fn validate_slug(name: &str) -> Result<()> {
    if name.is_empty() {
        return invalid(name, "must not be empty");
    }
    if name.len() > 64 {
        return invalid(name, "must be 64 characters or fewer");
    }
    let mut chars = name.chars();
    if !chars.next().is_some_and(|ch| ch.is_ascii_lowercase()) {
        return invalid(name, "must start with a lowercase ASCII letter");
    }
    if !chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_') {
        return invalid(
            name,
            "must contain only lowercase ASCII letters, digits, and underscores",
        );
    }
    Ok(())
}

fn invalid<T>(name: &str, reason: &str) -> Result<T> {
    Err(Error::InvalidName {
        name: name.to_string(),
        reason: reason.to_string(),
    })
}
