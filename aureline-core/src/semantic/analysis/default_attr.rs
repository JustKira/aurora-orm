use crate::ast::{Attribute, AttributeArg, AttributeValue, Field, Schema, SchemaItem, Type};

use super::{SemanticError, error};

pub(super) fn analyze(schema: &Schema, errors: &mut Vec<SemanticError>) {
    for item in &schema.items {
        let SchemaItem::TableDecl(table) = item else {
            continue;
        };

        for field in &table.fields {
            validate_field_defaults(field, errors);
        }
    }
}

fn validate_field_defaults(field: &Field, errors: &mut Vec<SemanticError>) {
    let mut seen_default = false;

    for attr in &field.raw_attributes {
        if attr.name != "default" {
            continue;
        }

        if seen_default {
            errors.push(err_at(
                attr,
                format!("@default on field `{}` is already defined", field.name),
            ));
            continue;
        }
        seen_default = true;

        if let Err(error) = validate_default_args(field, attr) {
            errors.push(error);
        }
    }
}

fn validate_default_args(field: &Field, attr: &Attribute) -> Result<(), SemanticError> {
    let mut positional_count = 0;

    for arg in &attr.args {
        match arg {
            AttributeArg::Keyword { name, value } if name == "always" => {
                if !matches!(value, AttributeValue::Bool { .. }) {
                    return Err(err_at(
                        attr,
                        "@default `always:` must be a boolean".to_string(),
                    ));
                }
            }
            AttributeArg::Keyword { name, .. } => {
                return Err(err_at(
                    attr,
                    format!("unknown @default arg `{name}`; expected `always`"),
                ));
            }
            AttributeArg::Positional { value } => {
                positional_count += 1;
                if positional_count > 1 {
                    return Err(err_at(
                        attr,
                        "@default expects exactly one positional value".to_string(),
                    ));
                }
                validate_default_value(value, attr)?;
                validate_default_value_type(field, value, attr)?;
            }
        }
    }

    if positional_count != 1 {
        return Err(err_at(
            attr,
            "@default expects exactly one positional value".to_string(),
        ));
    }

    Ok(())
}

fn validate_default_value_type(
    field: &Field,
    value: &AttributeValue,
    attr: &Attribute,
) -> Result<(), SemanticError> {
    if default_value_matches_type(value, &field.ty, field.optional) {
        return Ok(());
    }

    Err(err_at(
        attr,
        format!(
            "@default value type `{}` does not match field `{}` type `{}`",
            default_value_type_label(value),
            field.name,
            crate::emit::surql_type(&field.ty),
        ),
    ))
}

fn default_value_matches_type(value: &AttributeValue, ty: &Type, optional: bool) -> bool {
    if optional && matches!(value, AttributeValue::Ident { value } if value.eq_ignore_ascii_case("NONE")) {
        return true;
    }

    match (value, ty) {
        (_, Type::Primitive { name }) if name == "any" => true,
        (AttributeValue::String { .. }, Type::Primitive { name }) => name == "string",
        (AttributeValue::Bool { .. }, Type::Primitive { name }) => name == "bool",
        (AttributeValue::Number { value }, Type::Primitive { name }) => match name.as_str() {
            "int" => value.fract() == 0.0,
            "float" | "decimal" | "number" => true,
            _ => false,
        },
        (AttributeValue::Array { values }, Type::Array { inner, .. })
        | (AttributeValue::Array { values }, Type::Set { inner, .. }) => values
            .iter()
            .all(|value| default_value_matches_type(value, inner, false)),
        // SurQL expressions and bare identifiers may resolve to the right type at runtime.
        (AttributeValue::Surql { .. }, _) | (AttributeValue::Ident { .. }, _) => true,
        _ => false,
    }
}

fn default_value_type_label(value: &AttributeValue) -> &'static str {
    match value {
        AttributeValue::Number { .. } => "number",
        AttributeValue::Bool { .. } => "bool",
        AttributeValue::Ident { .. } => "identifier",
        AttributeValue::String { .. } => "string",
        AttributeValue::Surql { .. } => "surql",
        AttributeValue::Array { .. } => "array",
        AttributeValue::Tuple { .. } => "tuple",
    }
}

fn validate_default_value(value: &AttributeValue, attr: &Attribute) -> Result<(), SemanticError> {
    match value {
        AttributeValue::Surql { body, source_range } => {
            crate::surql::validate_expression(body).map_err(|error| SemanticError {
                message: error.message,
                hint: None,
                range: source_range.or(attr.source_range),
            })?;
        }
        AttributeValue::Array { values } | AttributeValue::Tuple { values } => {
            for value in values {
                validate_default_value(value, attr)?;
            }
        }
        AttributeValue::Number { .. }
        | AttributeValue::Bool { .. }
        | AttributeValue::Ident { .. }
        | AttributeValue::String { .. } => {}
    }
    Ok(())
}

fn err_at(attr: &Attribute, message: String) -> SemanticError {
    let mut err = error(message);
    err.range = attr.source_range;
    err
}
