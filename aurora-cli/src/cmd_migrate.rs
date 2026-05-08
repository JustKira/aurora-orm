use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use aurora_migrate::apply::{ApplyOpts, ApplyReport};
use aurora_migrate::config::{self, Config};
use aurora_migrate::diff::diff_schemas;
use aurora_migrate::error::Error;
use aurora_migrate::fs_io::{read_previous_schema, read_schema};
use aurora_migrate::journal::read_journal;
use aurora_migrate::{GenerateOpts, generate};

pub fn run(args: Vec<String>) -> Result<()> {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("generate") => generate_cmd(args.collect()),
        Some("diff") => diff_cmd(args.collect()),
        Some("apply") => apply_cmd(args.collect()),
        Some("help") | Some("--help") | Some("-h") | None => {
            print_help();
            Ok(())
        }
        Some(other) => bail!("unknown migrate command `{other}`\n\n{}", help_text()),
    }
}

fn generate_cmd(args: Vec<String>) -> Result<()> {
    let flags = parse_flags(args, FlagSet::generate())?;
    let (config, _config_dir) = load_config(flags.config.as_deref())?;
    let name = flags.name.context("missing --name <slug>")?;

    match generate(GenerateOpts {
        config,
        name,
        allow_empty: flags.allow_empty,
    }) {
        Ok(report) => {
            println!("Created {}", report.dir.display());
            print_ops(&report.ops);
            for warning in report.warnings {
                println!("warning: {warning}");
            }
            Ok(())
        }
        Err(Error::EmptyDiff) => {
            eprintln!("{}", Error::EmptyDiff);
            std::process::exit(2);
        }
        Err(error) => Err(error.into()),
    }
}

fn diff_cmd(args: Vec<String>) -> Result<()> {
    let flags = parse_flags(args, FlagSet::diff())?;
    let (config, _config_dir) = load_config(flags.config.as_deref())?;
    let new_schema = read_schema(&config.schema.file)?;
    let journal = read_journal(&config.migrations.dir.join("meta"))?;
    let previous = read_previous_schema(&config.migrations.dir.join("meta"), &journal)?;
    let ops = diff_schemas(&previous, &new_schema);
    print_ops(&ops);
    let destructive = ops.iter().filter(|op| op.destructive()).count();
    println!("{destructive} destructive change(s)");
    Ok(())
}

fn apply_cmd(args: Vec<String>) -> Result<()> {
    let flags = parse_flags(args, FlagSet::apply())?;
    let (config, config_dir) = load_config(flags.config.as_deref())?;

    let opts = ApplyOpts {
        config,
        config_dir,
        dry_run: flags.dry_run,
        allow_destructive: flags.allow_destructive,
    };

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("creating tokio runtime")?;
    let report = runtime.block_on(aurora_migrate::apply::apply(opts))?;
    print_apply_report(&report);
    Ok(())
}

/// Returns (config, dir-containing-aurora.toml). The dir is needed by the
/// applier to resolve relative `migrations.dir` and to locate `.env.local`.
fn load_config(path: Option<&Path>) -> Result<(Config, PathBuf)> {
    if let Some(path) = path {
        let config = config::load_file(path)?;
        let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        return Ok((config, dir));
    }
    let cwd = std::env::current_dir()?;
    Ok(config::load_with_dir(&cwd)?)
}

fn print_ops(ops: &[aurora_migrate::ops::Op]) {
    for op in ops {
        if op.destructive() {
            println!("  {} [DESTRUCTIVE]", op.summary());
        } else {
            println!("  {}", op.summary());
        }
    }
}

fn print_apply_report(report: &ApplyReport) {
    if report.dry_run {
        println!("(dry run — nothing changed on the database)");
    }
    if !report.skipped_already_applied.is_empty() {
        let list: Vec<String> = report
            .skipped_already_applied
            .iter()
            .map(|i| format!("{i:04}"))
            .collect();
        println!("Skipped (already applied): {}", list.join(", "));
    }
    if report.applied.is_empty() {
        println!("No new migrations to apply.");
    } else {
        println!("Applied:");
        for entry in &report.applied {
            let suffix = if entry.destructive { " [DESTRUCTIVE]" } else { "" };
            println!(
                "  {:04}_{}  ({} statements){}",
                entry.idx, entry.name, entry.statements, suffix
            );
        }
    }
}

struct Flags {
    name: Option<String>,
    config: Option<PathBuf>,
    allow_empty: bool,
    dry_run: bool,
    allow_destructive: bool,
}

#[derive(Default, Clone, Copy)]
struct FlagSet {
    name: bool,
    allow_empty: bool,
    dry_run: bool,
    allow_destructive: bool,
}

impl FlagSet {
    fn generate() -> Self {
        Self {
            name: true,
            allow_empty: true,
            ..Default::default()
        }
    }
    fn diff() -> Self {
        Self::default()
    }
    fn apply() -> Self {
        Self {
            dry_run: true,
            allow_destructive: true,
            ..Default::default()
        }
    }
}

fn parse_flags(args: Vec<String>, allowed: FlagSet) -> Result<Flags> {
    let mut flags = Flags {
        name: None,
        config: None,
        allow_empty: false,
        dry_run: false,
        allow_destructive: false,
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--name" if allowed.name => {
                flags.name = Some(args.next().context("missing --name value")?)
            }
            "--config" => {
                flags.config = Some(PathBuf::from(
                    args.next().context("missing --config value")?,
                ))
            }
            "--allow-empty" if allowed.allow_empty => flags.allow_empty = true,
            "--dry-run" if allowed.dry_run => flags.dry_run = true,
            "--allow-destructive" if allowed.allow_destructive => flags.allow_destructive = true,
            other => bail!("unknown flag `{other}`\n\n{}", help_text()),
        }
    }
    Ok(flags)
}

fn print_help() {
    print!("{}", help_text());
}

fn help_text() -> &'static str {
    "aurora migrate commands:\n  \
     aurora migrate generate --name <slug> [--config <path>] [--allow-empty]\n  \
     aurora migrate diff [--config <path>]\n  \
     aurora migrate apply [--config <path>] [--dry-run] [--allow-destructive]\n\n\
     Generated migrations do not detect renames; a rename is emitted as remove plus create.\n\n\
     `apply` reads SurrealDB connection from env vars; configure overrides in aurora.toml's\n\
     [database] block. See tools/aurora-migrate/src/apply for the env var name cascade.\n"
}
