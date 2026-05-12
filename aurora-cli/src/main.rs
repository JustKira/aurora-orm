use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use aurora_core::{check, parse_schema};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

mod cmd_migrate;
mod diagnostics;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".to_string());

    match command.as_str() {
        "parse" => {
            let path = required_path(args.next())?;
            let input = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let schema = parse_schema(&input)?;
            println!("{schema:#?}");
        }
        "check" => {
            let path = required_path(args.next())?;
            let input = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let report = check(&input);
            let mut writer = StandardStream::stderr(ColorChoice::Auto);
            for diagnostic in &report.diagnostics {
                diagnostics::emit_diagnostic(&mut writer, &path, &input, diagnostic)?;
            }
            if report.has_errors() {
                std::process::exit(1);
            }
            println!("OK: {} passed Aurora checks.", path.display());
        }
        "migrate" => cmd_migrate::run(args.collect())?,
        "help" | "--help" | "-h" => print_help(),
        other => bail!("unknown command `{other}`\n\n{}", help_text()),
    }

    Ok(())
}

fn required_path(value: Option<String>) -> Result<PathBuf> {
    value
        .map(PathBuf::from)
        .context("missing schema path; pass tools/aurora-examples/schema.aurora")
}

fn print_help() {
    print!("{}", help_text());
}

fn help_text() -> &'static str {
    "aurora internal language proof of concept\n\nCommands:\n  aurora parse <schema.aurora>\n  aurora check <schema.aurora>\n  aurora migrate generate --name <slug> [--config <path>] [--allow-empty]\n  aurora migrate diff [--config <path>]\n"
}
