use crate::ast::{Attribute, AttributeArg, AttributeValue, DefaultValue, Field};

use super::super::SemanticError;
use super::attributes::err_at;

pub(super) fn lower(field: &mut Field, attr: &Attribute, errors: &mut Vec<SemanticError>) {
    if field.default.is_some() {
        errors.push(err_at(
            attr,
            format!("@default on field `{}` is already defined", field.name),
        ));
        return;
    }
    match parse_default(attr) {
        Ok((value, always)) => {
            field.default = Some(value);
            field.always = always;
        }
        Err(error) => errors.push(error),
    }
}

fn parse_default(attr: &Attribute) -> Result<(DefaultValue, bool), SemanticError> {
    let mut default = None;
    let mut always = false;

    for arg in &attr.args {
        match arg {
            AttributeArg::Keyword { name, value } if name == "always" => {
                let AttributeValue::Bool { value } = value else {
                    return Err(err_at(
                        attr,
                        "@default `always:` must be a boolean".to_string(),
                    ));
                };
                always = *value;
            }
            AttributeArg::Keyword { name, .. } => {
                return Err(err_at(
                    attr,
                    format!("unknown @default arg `{name}`; expected `always`"),
                ));
            }
            AttributeArg::Positional { value } => {
                if default.is_some() {
                    return Err(err_at(
                        attr,
                        "@default expects exactly one positional value".to_string(),
                    ));
                }
                default = Some(default_value(value, attr)?);
            }
        }
    }

    let Some(default) = default else {
        return Err(err_at(
            attr,
            "@default expects exactly one positional value".to_string(),
        ));
    };

    Ok((default, always))
}

fn default_value(value: &AttributeValue, attr: &Attribute) -> Result<DefaultValue, SemanticError> {
    match value {
        AttributeValue::Number { value } => Ok(DefaultValue::Number { value: *value }),
        AttributeValue::Bool { value } => Ok(DefaultValue::Bool { value: *value }),
        AttributeValue::Ident { value } => Ok(DefaultValue::Ident {
            value: value.clone(),
        }),
        AttributeValue::String { value } => Ok(DefaultValue::String {
            value: value.clone(),
        }),
        AttributeValue::Surql { body, source_range } => {
            if let Err(error) = crate::surql::validate_expression(body) {
                return Err(SemanticError {
                    message: error.message,
                    hint: None,
                    range: source_range.or(attr.source_range),
                });
            }
            Ok(DefaultValue::Surql { body: body.clone() })
        }
        AttributeValue::Array { values } => values
            .iter()
            .map(|value| default_value(value, attr))
            .collect::<Result<Vec<_>, _>>()
            .map(|values| DefaultValue::Array { values }),
        AttributeValue::Tuple { values } => values
            .iter()
            .map(|value| default_value(value, attr))
            .collect::<Result<Vec<_>, _>>()
            .map(|values| DefaultValue::Tuple { values }),
    }
}
