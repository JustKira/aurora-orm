use std::collections::{HashMap, HashSet};

use crate::ast::{Schema, SchemaItem};

use super::context::{AnalysisContext, normalized_name};
use super::{SemanticError, error};

pub(super) fn analyze(
    schema: &Schema,
    _context: &AnalysisContext,
    errors: &mut Vec<SemanticError>,
) {
    table_names(schema, errors);
    analyzer_names(schema, errors);
    field_names(schema, errors);
}

fn table_names(schema: &Schema, errors: &mut Vec<SemanticError>) {
    // Raw duplicates are always invalid: two `table User` declarations are
    // ambiguous before we even think about emission.
    let mut seen_raw = HashSet::new();
    // Normalized duplicates are also invalid because both declarations would
    // emit to the same SurrealDB table name.
    let mut seen_normalized = HashMap::new();

    for item in &schema.items {
        let SchemaItem::TableDecl(table) = item else {
            continue;
        };

        if !seen_raw.insert(table.name.as_str()) {
            let mut err = error(format!("duplicate table name `{}`", table.name));
            err.range = table.name_range;
            errors.push(err);
        }

        let normalized = normalized_name(&table.name);
        if let Some(previous_raw) = seen_normalized.insert(normalized.clone(), table.name.as_str())
        {
            // Exact duplicate raw names are already reported above. This branch
            // is specifically for collisions like `User` vs `user`.
            if previous_raw != table.name {
                let mut err = error(format!(
                    "duplicate table name `{normalized}` after normalization"
                ));
                err.range = table.name_range;
                errors.push(err);
            }
        }
    }
}

fn analyzer_names(schema: &Schema, errors: &mut Vec<SemanticError>) {
    let mut seen = HashSet::new();

    for item in &schema.items {
        let SchemaItem::AnalyzerDecl(analyzer) = item else {
            continue;
        };

        if !seen.insert(analyzer.name.as_str()) {
            let mut err = error(format!("duplicate analyzer name `{}`", analyzer.name));
            err.range = analyzer.name_range;
            errors.push(err);
        }
    }
}

fn field_names(schema: &Schema, errors: &mut Vec<SemanticError>) {
    for item in &schema.items {
        let SchemaItem::TableDecl(table) = item else {
            continue;
        };

        // Fields are scoped to their table. Reusing `id` on different tables is
        // valid; declaring it twice inside the same table is not.
        let mut seen = HashSet::new();
        for field in &table.fields {
            if !seen.insert(field.name.as_str()) {
                let mut err = error(format!(
                    "duplicate field name `{}` on table {}",
                    field.name, table.name
                ));
                err.range = field.name_range;
                errors.push(err);
            }
        }
    }
}
