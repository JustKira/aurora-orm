use std::collections::BTreeSet;

use crate::ast::{Attribute, AttributeArg, AttributeValue, Function, Schema, SchemaItem};

use super::super::AttributeScope;
use super::super::diagnostics::unknown_attribute;
use super::{SemanticError, error};

const FUNCTION_ATTRS: &[&str] = &["allow"];

pub(super) fn analyze(schema: &Schema, errors: &mut Vec<SemanticError>) {
    for item in &schema.items {
        let SchemaItem::FunctionDecl(function) = item else {
            continue;
        };

        validate_reserved_params(function, errors);
        validate_body_params(function, errors);
        validate_attributes(function, errors);
    }
}

fn validate_reserved_params(function: &Function, errors: &mut Vec<SemanticError>) {
    for param in &function.params {
        if crate::surql::is_builtin_param(&param.name) {
            let mut err = error(format!(
                "function parameter name `{}` is reserved",
                param.name
            ));
            err.range = param.name_range.or(function.source_range);
            errors.push(err);
        }
    }
}

fn validate_body_params(function: &Function, errors: &mut Vec<SemanticError>) {
    let declared = function
        .params
        .iter()
        .filter(|param| !crate::surql::is_builtin_param(&param.name))
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();

    let referenced = match crate::surql::function_body_params(&function.body.body) {
        Ok(referenced) => referenced,
        Err(parse_error) => {
            let mut err = error(parse_error.message);
            err.range = function.source_range;
            errors.push(err);
            return;
        }
    };

    let missing = declared
        .difference(&referenced)
        .cloned()
        .collect::<Vec<_>>();
    let unknown = referenced
        .difference(&declared)
        .cloned()
        .collect::<Vec<_>>();

    if missing.is_empty() && unknown.is_empty() {
        return;
    }

    let mut parts = Vec::new();
    if !missing.is_empty() {
        parts.push(format!(
            "missing references for function arguments: {}",
            missing
                .iter()
                .map(|name| format!("`${name}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !unknown.is_empty() {
        parts.push(format!(
            "unknown function body parameters: {}",
            unknown
                .iter()
                .map(|name| format!("`${name}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let mut err = error(format!(
        "function `{}` SurQL body parameters do not match signature: {}",
        function.name,
        parts.join("; ")
    ));
    err.range = function.source_range;
    errors.push(err);
}

fn validate_attributes(function: &Function, errors: &mut Vec<SemanticError>) {
    for attr in &function.raw_attributes {
        match attr.name.as_str() {
            "allow" => {
                if let Err(error) = validate_allow_args(attr) {
                    errors.push(error);
                }
            }
            unknown => {
                errors.push(
                    unknown_attribute(AttributeScope::FunctionBlock, unknown, FUNCTION_ATTRS)
                        .at(attr.source_range),
                );
            }
        }
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
                        "@@allow has duplicate `op:` arguments".to_string(),
                    ));
                }
                let AttributeValue::String { value } = value else {
                    return Err(err_at(
                        attr,
                        "@@allow `op:` must be a string literal like \"RUN\"".to_string(),
                    ));
                };
                operation = Some(value.as_str());
            }
            AttributeArg::Keyword { name, .. } => {
                return Err(err_at(
                    attr,
                    format!("unknown @@allow arg `{name}`; expected `op`"),
                ));
            }
            AttributeArg::Positional {
                value: AttributeValue::Surql { body, source_range },
            } => {
                if permission.is_some() {
                    return Err(err_at(
                        attr,
                        "@@allow has duplicate `#surql { ... }` permission blocks".to_string(),
                    ));
                }
                permission = Some((body, source_range));
            }
            AttributeArg::Positional { .. } => {
                return Err(err_at(
                    attr,
                    "@@allow positional arguments must be `#surql { ... }`; use `op: \"RUN\"` for the operation".to_string(),
                ));
            }
        }
    }

    let Some(operation) = operation else {
        return Err(err_at(
            attr,
            "@@allow requires an `op: \"RUN\"` argument".to_string(),
        ));
    };

    if !operation.eq_ignore_ascii_case("RUN") {
        return Err(err_at(
            attr,
            format!("unknown @@allow operation `{operation}`; expected RUN"),
        ));
    }

    let Some((body, source_range)) = permission else {
        return Err(err_at(
            attr,
            "@@allow requires one positional `#surql { ... }` permission block".to_string(),
        ));
    };

    crate::surql::validate_function_permission(body).map_err(|error| SemanticError {
        message: error.message,
        hint: None,
        range: source_range.or(attr.source_range),
    })
}

fn err_at(attr: &Attribute, message: String) -> SemanticError {
    let mut err = error(message);
    err.range = attr.source_range;
    err
}
