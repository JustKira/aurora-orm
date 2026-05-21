//! Non-mutating semantic checks over the raw AST.
//!
//! This module intentionally does not lower attributes. It collects the schema
//! symbols once, then runs meaning checks that need a whole-schema view.

mod analyzers;
mod context;
mod functions;
mod surql;
mod symbols;
mod types;

use crate::ast::Schema;
use crate::schema_index::SchemaIndex;

use super::SemanticResult;
use context::AnalysisContext;

/// Meaning-only checks over the raw AST.
///
/// This stage should not mutate the schema. It is where global rules belong:
/// duplicate symbols, record table resolution, analyzer references, top-level
/// SurQL syntax, and escape-hatch variable scope.
pub fn analyze(schema: &Schema) -> SemanticResult {
    // Build shared lookup tables once so each semantic pass can focus on one
    // rule family instead of repeatedly walking the schema for common symbols.
    let context = AnalysisContext::new(schema);
    let schema_index = SchemaIndex::from_schema(schema);
    let mut errors = Vec::new();

    // Keep the passes small and meaning-oriented. If one of these files grows
    // large, split it under a folder with the same name.
    symbols::analyze(schema, &context, &mut errors);
    types::analyze(schema, &context, &mut errors);
    analyzers::analyze(schema, &schema_index, &mut errors);
    functions::analyze(schema, &mut errors);
    surql::analyze(schema, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
