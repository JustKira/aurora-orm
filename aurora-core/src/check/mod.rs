pub mod diagnostics;

mod classifiers;
mod context;
mod keywords;
mod range;
mod report;
mod rules;
pub(crate) mod syntax;

pub use report::CheckReport;

use diagnostics::{Diagnostic, DiagnosticCode, SourceRange};

pub fn check(source: &str) -> CheckReport {
    let mut schema = match crate::parse_to_ast(source) {
        Ok(schema) => schema,
        Err(crate::AuroraError::Parse(diagnostic)) => {
            return CheckReport {
                schema: None,
                diagnostics: vec![diagnostic],
            };
        }
        Err(error) => {
            return CheckReport {
                schema: None,
                diagnostics: vec![Diagnostic::error(
                    DiagnosticCode::ConvertError,
                    error.to_string(),
                    SourceRange::first_character(),
                )],
            };
        }
    };

    match crate::validate::validate(&mut schema) {
        Ok(()) => CheckReport::ok(schema),
        Err(errors) => CheckReport {
            schema: Some(schema),
            diagnostics: errors
                .into_iter()
                .map(|error| {
                    let mut diagnostic = Diagnostic::error(
                        DiagnosticCode::ValidationError,
                        error.message,
                        SourceRange::first_character(),
                    );
                    if let Some(hint) = error.hint {
                        diagnostic = diagnostic.with_help(hint);
                    }
                    diagnostic
                })
                .collect(),
        },
    }
}
