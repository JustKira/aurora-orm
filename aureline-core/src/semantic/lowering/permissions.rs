use crate::ast::{Attribute, AttributeArg, AttributeValue};

use super::super::SemanticError;
use super::attributes::err_at;

pub(super) fn lower(attr: &Attribute, errors: &mut Vec<SemanticError>) {
    if let Err(error) = validate_allow_args(attr) {
        errors.push(error);
    }
}

fn validate_allow_args(attr: &Attribute) -> Result<(), SemanticError> {
    let mut operation = None;
    let mut permission = None;

    for arg in &attr.args {
        match arg {
            AttributeArg::Keyword { name, value } if name == "op" => {
                if operation.is_some() {
                    return Err(err_at(
                        attr,
                        "@allow has duplicate `op:` arguments".to_string(),
                    ));
                }
                let AttributeValue::String { value } = value else {
                    return Err(err_at(
                        attr,
                        "@allow `op:` must be a string literal like \"SELECT\"".to_string(),
                    ));
                };
                operation = Some(value.as_str());
            }
            AttributeArg::Keyword { name, .. } => {
                return Err(err_at(
                    attr,
                    format!("unknown @allow arg `{name}`; expected `op`"),
                ));
            }
            AttributeArg::Positional {
                value: AttributeValue::Surql { body, source_range },
            } => {
                if permission.is_some() {
                    return Err(err_at(
                        attr,
                        "@allow has duplicate `#surql { ... }` permission blocks".to_string(),
                    ));
                }
                permission = Some((body, source_range));
            }
            AttributeArg::Positional { .. } => {
                return Err(err_at(
                    attr,
                    "@allow positional arguments must be `#surql { ... }`; use `op: \"SELECT\"` for the operation".to_string(),
                ));
            }
        }
    }

    let Some(operation) = operation else {
        return Err(err_at(
            attr,
            "@allow requires an `op: \"SELECT\"` argument".to_string(),
        ));
    };

    let Some((body, source_range)) = permission else {
        return Err(err_at(
            attr,
            "@allow requires one positional `#surql { ... }` permission block".to_string(),
        ));
    };

    let operation_keyword = operation.to_ascii_uppercase();
    match operation_keyword.as_str() {
        "SELECT" | "CREATE" | "UPDATE" | "DELETE" => {}
        _ => {
            return Err(err_at(
                attr,
                format!(
                    "unknown @allow operation `{operation}`; expected one of: SELECT, CREATE, UPDATE, DELETE"
                ),
            ));
        }
    }

    crate::surql::validate_field_permission(&operation_keyword, body).map_err(|error| {
        SemanticError {
            message: error.message,
            hint: None,
            range: source_range.or(attr.source_range),
        }
    })
}
