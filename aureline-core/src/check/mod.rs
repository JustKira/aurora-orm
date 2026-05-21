pub mod diagnostics;

mod context;
mod keywords;
mod range;
mod recovery;
mod report;
mod rules;
pub(crate) mod syntax;

pub use report::CheckReport;

use diagnostics::{Diagnostic, DiagnosticCode, SourceRange};
use recovery::recovery_diagnostics;

pub fn check(source: &str) -> CheckReport {
    let pairs = match crate::parse_source_file(source) {
        Ok(pairs) => pairs,
        Err(error) => {
            let diagnostic = crate::check::syntax::parse_diagnostic_from_pest(error);
            return CheckReport {
                schema: None,
                diagnostics: vec![diagnostic],
            };
        }
    };
    let mut diagnostics = recovery_diagnostics(pairs.clone());
    let mut schema = match crate::parse_source_file_pairs_to_ast(pairs) {
        Ok(schema) => schema,
        Err(error) => {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::ConvertError,
                error.to_string(),
                SourceRange::first_character(),
            ));
            return CheckReport {
                schema: None,
                diagnostics,
            };
        }
    };

    match crate::semantic::validate(&mut schema) {
        Ok(()) => CheckReport {
            schema: Some(schema),
            diagnostics,
        },
        Err(errors) => {
            diagnostics.extend(errors.into_iter().map(|error| {
                let mut diagnostic = Diagnostic::error(
                    DiagnosticCode::ValidationError,
                    error.message(),
                    error.range().unwrap_or_else(SourceRange::first_character),
                );
                if let Some(hint) = error.hint() {
                    diagnostic = diagnostic.with_help(hint);
                }
                diagnostic
            }));
            CheckReport {
                schema: Some(schema),
                diagnostics,
            }
        }
    }
}
