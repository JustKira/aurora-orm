//! Mutating semantic steps.
//!
//! Lowering converts parser-only syntax into checked schema data used by
//! emitters and migration. Analysis has already validated cross-schema meaning;
//! this phase interprets attributes and updates the AST in place.

mod assertions;
mod attributes;
mod flexible;
mod fulltext;
mod hnsw;
mod indexes;
mod permissions;

use crate::ast::Schema;

use super::SemanticResult;

pub fn lower(schema: &mut Schema) -> SemanticResult {
    attributes::lower(schema)
}
