use std::fs;
use std::path::{Path, PathBuf};

use aureline_core::ast::Schema;
use chrono::Utc;

use crate::checksum::sha256_hex;
use crate::config::Config;
use crate::error::{Error, Result, io};
use crate::fs_io::{read_previous_schema, read_schema, write_atomic};
use crate::journal::{Journal, JournalEntry, append_entry, next_idx, read_journal, validate_slug};
use crate::ops::Op;
use crate::plan::{MigrationPlan, plan_changes};
use crate::render::{emit_down, emit_up};
use crate::schema::full_schema;
use crate::snapshot::canonicalize;

pub struct GenerateOpts {
    pub config: Config,
    pub name: String,
    pub allow_empty: bool,
}

pub struct GenerateReport {
    pub idx: u32,
    pub name: String,
    pub dir: PathBuf,
    pub ops: Vec<Op>,
    pub destructive_ops: Vec<usize>,
    pub warnings: Vec<String>,
}

pub fn generate(opts: GenerateOpts) -> Result<GenerateReport> {
    validate_slug(&opts.name)?;
    let paths = Paths::from_config(&opts.config);

    let outcome = compute_plan(&paths)?;
    if outcome.plan.steps.is_empty() && !opts.allow_empty {
        return Err(Error::EmptyDiff);
    }

    let idx = next_idx(&outcome.journal);
    let dir = paths.migration_dir(idx, &opts.name);
    let up = write_artifacts(&paths, &dir, idx, &outcome.plan.steps, &outcome.new_schema)?;

    let report = build_report(idx, opts.name, dir, outcome.plan);
    record_in_journal(&paths.meta_dir, &report, &up)?;
    Ok(report)
}

struct Paths {
    schema: PathBuf,
    migrations_dir: PathBuf,
    meta_dir: PathBuf,
}

impl Paths {
    fn from_config(c: &Config) -> Self {
        let migrations_dir = c.migrations.dir.clone();
        let meta_dir = migrations_dir.join("meta");
        Self {
            schema: c.schema.file.clone(),
            migrations_dir,
            meta_dir,
        }
    }

    fn migration_dir(&self, idx: u32, name: &str) -> PathBuf {
        self.migrations_dir.join(format!("{idx:04}_{name}"))
    }
}

struct PlanOutcome {
    plan: MigrationPlan,
    new_schema: Schema,
    journal: Journal,
}

/// Loads the new schema and the previous snapshot off disk, then diffs them.
/// Returns the journal too so the caller can pick the next index without re-reading it.
fn compute_plan(paths: &Paths) -> Result<PlanOutcome> {
    let new_schema = full_schema(&read_schema(&paths.schema)?);
    let journal = read_journal(&paths.meta_dir)?;
    let prev = full_schema(&read_previous_schema(&paths.meta_dir, &journal)?);
    let changes = crate::diff::diff_changes(&prev, &new_schema);
    let plan = plan_changes(changes);
    Ok(PlanOutcome {
        plan,
        new_schema,
        journal,
    })
}

/// Writes migration.surql, down.surql, the new snapshot, and ensures the lockfile.
/// Returns the up-SQL so the caller can checksum it for the journal entry.
fn write_artifacts(
    paths: &Paths,
    dir: &Path,
    idx: u32,
    ops: &[Op],
    new_schema: &Schema,
) -> Result<String> {
    fs::create_dir_all(dir).map_err(|error| io(dir, error))?;
    fs::create_dir_all(&paths.meta_dir).map_err(|error| io(&paths.meta_dir, error))?;

    let up = emit_up(ops);
    let down = emit_down(ops);
    // Forward migration — SurrealQL applied when moving the schema to this version.
    write_atomic(&dir.join("migration.surql"), up.as_bytes())?;
    // Reverse migration — SurrealQL applied when rolling back from this version.
    write_atomic(&dir.join("down.surql"), down.as_bytes())?;
    // Frozen snapshot of the new schema — becomes the "previous" baseline for the next diff.
    write_atomic(
        &paths.meta_dir.join(format!("{idx:04}_snapshot.json")),
        canonicalize(new_schema).as_bytes(),
    )?;
    ensure_compatible_lock(&paths.migrations_dir)?;
    Ok(up)
}

fn build_report(idx: u32, name: String, dir: PathBuf, plan: MigrationPlan) -> GenerateReport {
    let destructive_ops = plan
        .risks
        .iter()
        .filter_map(|risk| risk.blocks_by_default().then_some(risk.step_index))
        .collect::<Vec<_>>();
    let warnings = plan
        .risks
        .iter()
        .map(|risk| risk.message.clone())
        .collect::<Vec<_>>();
    GenerateReport {
        idx,
        name,
        dir,
        ops: plan.steps,
        destructive_ops,
        warnings,
    }
}

fn record_in_journal(meta_dir: &Path, report: &GenerateReport, up: &str) -> Result<()> {
    append_entry(
        meta_dir,
        JournalEntry {
            idx: report.idx,
            name: report.name.clone(),
            created_at: Utc::now(),
            checksum: sha256_hex(up.as_bytes()),
            destructive: !report.destructive_ops.is_empty(),
            warnings: report.warnings.clone(),
            kind: "generated".to_string(),
        },
    )
}

fn ensure_compatible_lock(migrations_dir: &Path) -> Result<()> {
    let path = migrations_dir.join("migration_lock.toml");
    if path.exists() {
        let contents = fs::read_to_string(&path).map_err(|error| io(&path, error))?;
        let lock = toml::from_str::<MigrationLock>(&contents).map_err(|error| Error::Lock {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
        if lock.provider != "surrealdb" {
            return Err(Error::Lock {
                path: path.display().to_string(),
                message: format!("unsupported provider '{}'", lock.provider),
            });
        }
        if lock.snapshot_version != crate::snapshot::SNAPSHOT_VERSION {
            return Err(Error::Lock {
                path: path.display().to_string(),
                message: format!(
                    "unsupported snapshot_version {}; expected {}",
                    lock.snapshot_version,
                    crate::snapshot::SNAPSHOT_VERSION
                ),
            });
        }
        return Ok(());
    }
    write_atomic(
        &path,
        format!(
            "provider = \"surrealdb\"\nsnapshot_version = {}\n",
            crate::snapshot::SNAPSHOT_VERSION
        )
        .as_bytes(),
    )
}

#[derive(serde::Deserialize)]
struct MigrationLock {
    provider: String,
    snapshot_version: u32,
}
