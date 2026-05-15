use std::collections::BTreeSet;

use crate::ast::{Attribute, AttributeArg, AttributeValue, Function, Schema, SchemaItem};

use super::{SemanticError, error};

const FUNCTION_ATTRS: &[&str] = &["allow"];

pub(super) fn analyze(schema: &Schema, errors: &mut Vec<SemanticError>) {
    for item in &schema.items {
        let SchemaItem::FunctionDecl(function) = item else {
            continue;
        };

        validate_body_params(function, errors);
        validate_attributes(function, errors);
    }
}

fn validate_body_params(function: &Function, errors: &mut Vec<SemanticError>) {
    let declared = function
        .params
        .iter()
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
                let mut err = unknown_attribute(unknown, FUNCTION_ATTRS, "function block");
                err.range = attr.source_range;
                errors.push(err);
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
                value: AttributeValue::Surql { body, .. },
            } => {
                if permission.is_some() {
                    return Err(err_at(
                        attr,
                        "@@allow has duplicate `#surql { ... }` permission blocks".to_string(),
                    ));
                }
                permission = Some(body);
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

    if permission.is_none() {
        return Err(err_at(
            attr,
            "@@allow requires one positional `#surql { ... }` permission block".to_string(),
        ));
    }

    Ok(())
}

fn err_at(attr: &Attribute, message: String) -> SemanticError {
    let mut err = error(message);
    err.range = attr.source_range;
    err
}

fn unknown_attribute(name: &str, valid: &[&str], scope: &str) -> SemanticError {
    let suggestion = closest_match(name, valid);
    SemanticError {
        message: format!("unknown {scope} attribute `@@{name}`"),
        hint: suggestion.map(|s| format!("did you mean `@@{s}`?")),
        range: None,
    }
}

fn closest_match(target: &str, candidates: &[&str]) -> Option<String> {
    let mut best: Option<(&str, usize)> = None;
    for candidate in candidates {
        let distance = levenshtein(target, candidate);
        if distance <= 3 && best.is_none_or(|(_, best_distance)| distance < best_distance) {
            best = Some((candidate, distance));
        }
    }
    best.map(|(candidate, _)| candidate.to_string())
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a = a.chars().collect::<Vec<_>>();
    let b = b.chars().collect::<Vec<_>>();
    let mut prev = (0..=b.len()).collect::<Vec<_>>();
    let mut curr = vec![0; b.len() + 1];

    for (i, ac) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, bc) in b.iter().enumerate() {
            let cost = usize::from(ac != bc);
            curr[j + 1] = (curr[j] + 1).min(prev[j + 1] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b.len()]
}
