//! Semantic layer between the parser and user-facing diagnostics.
//!
//! The parser produces a raw AST. The semantic layer is responsible for making
//! that AST meaningful: first by analyzing cross-schema rules, then by lowering
//! raw attributes into the checked schema representation consumed by emitters
//! and migration.

mod analysis;
mod diagnostics;
mod error;
mod lowering;

use crate::ast::Schema;

pub use error::{
    AttributeScope, SemanticDiagnostic, SemanticDiagnosticKind, SemanticError, SemanticResult,
};

pub use analysis::analyze;
pub use lowering::lower;

/// Analyze and lower a raw AST into a checked schema.
pub fn validate(schema: &mut Schema) -> SemanticResult {
    let mut errors = Vec::new();

    // Analysis is read-only and runs against the raw AST. It catches meaning
    // errors that do not require mutating attributes, such as duplicate symbols
    // and unresolved references.
    if let Err(mut semantic_errors) = analyze(schema) {
        errors.append(&mut semantic_errors);
    }
    // Lowering is the mutating step. It interprets raw attributes and fills the
    // checked schema fields (`indexes`, `flexible`, etc.) that emit/migrate use.
    if let Err(mut lowering_errors) = lower(schema) {
        errors.append(&mut lowering_errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
