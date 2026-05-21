use crate::ast::{AttributeArg, AttributeValue, Schema, SchemaItem};
use crate::schema_index::SchemaIndex;

use super::super::SemanticError;
use super::super::diagnostics::unknown_analyzer;

pub(super) fn analyze(
    schema: &Schema,
    schema_index: &SchemaIndex<'_>,
    errors: &mut Vec<SemanticError>,
) {
    for item in &schema.items {
        let SchemaItem::TableDecl(table) = item else {
            continue;
        };

        for field in &table.fields {
            for attr in &field.raw_attributes {
                if attr.name != "fulltext" {
                    continue;
                }

                // Attribute-shape validation still belongs to lowering. Here
                // we only resolve a well-formed `analyzer:` reference if one is
                // present, avoiding duplicate errors for malformed attributes.
                let Some(analyzer) = fulltext_analyzer_name(attr.args.as_slice()) else {
                    continue;
                };

                if !schema_index.has_analyzer(analyzer) {
                    errors.push(unknown_analyzer(analyzer).at(attr.source_range));
                }
            }
        }
    }
}

fn fulltext_analyzer_name(args: &[AttributeArg]) -> Option<&str> {
    // Only `analyzer: ident` is meaningful for symbol resolution. Other forms
    // are reported by the fulltext attribute parser during lowering.
    args.iter().find_map(|arg| match arg {
        AttributeArg::Keyword {
            name,
            value: AttributeValue::Ident { value },
        } if name == "analyzer" => Some(value.as_str()),
        _ => None,
    })
}
