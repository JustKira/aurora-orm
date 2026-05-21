use crate::ast::{Attribute, AttributeArg, AttributeValue};

use super::super::SemanticError;
use super::super::diagnostics::invalid_attribute_usage;
use super::attributes::err_at;

pub(super) fn lower(attr: &Attribute, errors: &mut Vec<SemanticError>) {
    match attr.args.as_slice() {
        [
            AttributeArg::Positional {
                value: AttributeValue::Surql { body, source_range },
            },
        ] => {
            if let Err(error) = crate::surql::validate_expression(body) {
                errors.push(
                    invalid_attribute_usage(error.message).at(source_range.or(attr.source_range)),
                );
            }
        }
        _ => {
            errors.push(err_at(
                attr,
                "@assert expects exactly one `#surql { ... }` block".to_string(),
            ));
        }
    }
}
