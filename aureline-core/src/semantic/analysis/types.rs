use crate::ast::{Schema, SchemaItem, Type};

use super::super::SemanticError;
use super::super::diagnostics::unknown_record_table;
use super::context::AnalysisContext;

pub(super) fn analyze(schema: &Schema, context: &AnalysisContext, errors: &mut Vec<SemanticError>) {
    for item in &schema.items {
        match item {
            SchemaItem::TableDecl(table) => {
                for field in &table.fields {
                    // Type references can be nested (`array<record<User>>`), so check
                    // the full tree instead of only the top-level field type.
                    record_references(&field.ty, context, errors);
                }
            }
            SchemaItem::FunctionDecl(function) => {
                for param in &function.params {
                    record_references(&param.ty, context, errors);
                }
                record_references(&function.return_type, context, errors);
            }
            SchemaItem::DocComment { .. } | SchemaItem::AnalyzerDecl(_) => {}
        }
    }
}

fn record_references(ty: &Type, context: &AnalysisContext, errors: &mut Vec<SemanticError>) {
    match ty {
        // Bare `record` has no target table and is handled by the fallback arm.
        // Only constrained records need symbol resolution.
        Type::Record { table: Some(table) } => {
            check_record_table_exists(table, context, errors);
        }
        Type::Option { inner } | Type::Array { inner, .. } | Type::Set { inner, .. } => {
            record_references(inner, context, errors);
        }
        Type::Primitive { .. } | Type::Record { table: None } | Type::Geometry { .. } => {}
    }
}

fn check_record_table_exists(
    table: &str,
    context: &AnalysisContext,
    errors: &mut Vec<SemanticError>,
) {
    if !context.has_table(table) {
        errors.push(unknown_record_table(table));
    }
}
