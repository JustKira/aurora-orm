use crate::ast::{AttributeArg, AttributeValue, Schema, SchemaItem};

use super::context::AnalysisContext;
use super::{SemanticError, error};

pub(super) fn analyze(schema: &Schema, context: &AnalysisContext, errors: &mut Vec<SemanticError>) {
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

                if !context.has_analyzer(analyzer) {
                    let mut err = error(format!("unknown analyzer `{analyzer}`"));
                    err.range = attr.source_range;
                    errors.push(err);
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
