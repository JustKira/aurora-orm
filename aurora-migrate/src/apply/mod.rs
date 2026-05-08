//! Apply Aurora migrations against a live SurrealDB instance.
//!
//! The applier reads `migrations/meta/_journal.json` (the canonical list of
//! migrations on disk), connects to SurrealDB using env-var-based credentials
//! per `aurora.toml`'s `[database]` block, and runs each migration that
//! hasn't yet been recorded in the `_aurora_migrations` tracking table on
//! the target database.

mod client;
mod env;
mod tracking;

use std::path::{Path, PathBuf};

use aurora_config::Config;

use crate::journal::{Journal, JournalEntry, read_journal};

pub use client::{Client, ClientError};
pub use env::{Connection, EnvError, resolve_connection};
pub use tracking::{AppliedRecord, ensure_tracking_table, read_applied, record_applied};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Migrate(#[from] crate::error::Error),
    #[error("env: {0}")]
    Env(#[from] EnvError),
    #[error("client: {0}")]
    Client(#[from] ClientError),
    #[error(
        "migration {idx:04}_{name} would drop or invalidate data; pass --allow-destructive to proceed"
    )]
    Destructive { idx: u32, name: String },
    #[error(
        "migration {idx:04}_{name} on disk has checksum {disk}, but the journal records {journal}; refusing to apply"
    )]
    ChecksumDrift {
        idx: u32,
        name: String,
        disk: String,
        journal: String,
    },
    #[error(
        "applied migration {idx:04}_{name} has checksum {applied}, but the local journal records {journal}; refusing to proceed"
    )]
    AppliedChecksumDrift {
        idx: u32,
        name: String,
        applied: String,
        journal: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct ApplyOpts {
    pub config: Config,
    pub config_dir: PathBuf,
    pub dry_run: bool,
    pub allow_destructive: bool,
}

#[derive(Debug, Default)]
pub struct ApplyReport {
    pub applied: Vec<AppliedSummary>,
    pub skipped_already_applied: Vec<u32>,
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct AppliedSummary {
    pub idx: u32,
    pub name: String,
    pub destructive: bool,
    pub statements: usize,
}

pub async fn apply(opts: ApplyOpts) -> Result<ApplyReport> {
    let migrations_dir = opts.config_dir.join(&opts.config.migrations.dir);
    let meta_dir = migrations_dir.join("meta");
    let journal = read_journal(&meta_dir)?;

    if journal.entries.is_empty() {
        return Ok(ApplyReport {
            dry_run: opts.dry_run,
            ..Default::default()
        });
    }

    let connection = resolve_connection(&opts.config.database, &opts.config_dir)?;
    let client = Client::connect(connection).await?;

    ensure_tracking_table(&client).await?;
    let already_applied = read_applied(&client).await?;

    let mut report = ApplyReport {
        dry_run: opts.dry_run,
        ..Default::default()
    };

    for entry in &journal.entries {
        if let Some(applied) = already_applied.iter().find(|a| a.idx == entry.idx) {
            // Drift check: applied checksum should match the journal's. If
            // someone hand-edited a migration after applying it, refuse.
            if applied.checksum != entry.checksum {
                return Err(Error::AppliedChecksumDrift {
                    idx: entry.idx,
                    name: entry.name.clone(),
                    applied: applied.checksum.clone(),
                    journal: entry.checksum.clone(),
                });
            }
            report.skipped_already_applied.push(entry.idx);
            continue;
        }

        let folder = migrations_dir.join(format!("{:04}_{}", entry.idx, entry.name));
        let migration_path = folder.join("migration.surql");
        let sql = read_migration(&migration_path)?;

        verify_disk_checksum(entry, &sql)?;
        if entry.destructive && !opts.allow_destructive {
            return Err(Error::Destructive {
                idx: entry.idx,
                name: entry.name.clone(),
            });
        }

        let statements = count_statements(&sql);
        if !opts.dry_run {
            client.run_sql(&sql).await?;
            record_applied(&client, entry).await?;
        }

        report.applied.push(AppliedSummary {
            idx: entry.idx,
            name: entry.name.clone(),
            destructive: entry.destructive,
            statements,
        });
    }

    Ok(report)
}

fn read_migration(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|source| Error::Io {
        path: path.display().to_string(),
        source,
    })
}

fn verify_disk_checksum(entry: &JournalEntry, sql: &str) -> Result<()> {
    let disk = crate::checksum::sha256_hex(sql.as_bytes());
    if disk != entry.checksum {
        return Err(Error::ChecksumDrift {
            idx: entry.idx,
            name: entry.name.clone(),
            disk,
            journal: entry.checksum.clone(),
        });
    }
    Ok(())
}

/// Rough count of `;`-terminated statements; used only for reporting.
fn count_statements(sql: &str) -> usize {
    sql.lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with("--"))
        .filter(|line| line.trim_end().ends_with(';'))
        .count()
}

/// Re-export for callers that need the canonical journal type.
pub fn open_journal(meta_dir: &Path) -> crate::error::Result<Journal> {
    read_journal(meta_dir)
}
