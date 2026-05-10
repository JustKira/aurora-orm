use super::diagnostics::{Diagnostic, Severity};
use crate::Schema;

#[derive(Debug, Clone, PartialEq)]
pub struct CheckReport {
    pub schema: Option<Schema>,
    pub diagnostics: Vec<Diagnostic>,
}

impl CheckReport {
    pub fn ok(schema: Schema) -> Self {
        Self {
            schema: Some(schema),
            diagnostics: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }
}
