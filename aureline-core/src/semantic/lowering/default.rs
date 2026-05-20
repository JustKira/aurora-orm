use crate::ast::{Attribute, AttributeArg, AttributeValue, DefaultValue, Field};

pub(super) fn lower(field: &mut Field, attr: &Attribute) {
    match parse_default(attr) {
        Ok((value, always)) => {
            field.default = Some(value);
            field.always = always;
        }
        Err(()) => {}
    }
}

fn parse_default(attr: &Attribute) -> Result<(DefaultValue, bool), ()> {
    let mut default = None;
    let mut always = false;

    for arg in &attr.args {
        match arg {
            AttributeArg::Keyword { name, value } if name == "always" => {
                if let AttributeValue::Bool { value } = value {
                    always = *value;
                } else {
                    return Err(());
                };
            }
            AttributeArg::Keyword { .. } => return Err(()),
            AttributeArg::Positional { value } => {
                if default.is_some() {
                    return Err(());
                }
                default = Some(default_value(value)?);
            }
        }
    }

    let Some(default) = default else { return Err(()) };

    Ok((default, always))
}

fn default_value(value: &AttributeValue) -> Result<DefaultValue, ()> {
    match value {
        AttributeValue::Number { value } => Ok(DefaultValue::Number { value: *value }),
        AttributeValue::Bool { value } => Ok(DefaultValue::Bool { value: *value }),
        AttributeValue::Ident { value } => Ok(DefaultValue::Ident {
            value: value.clone(),
        }),
        AttributeValue::String { value } => Ok(DefaultValue::String {
            value: value.clone(),
        }),
        AttributeValue::Surql { body, .. } => Ok(DefaultValue::Surql { body: body.clone() }),
        AttributeValue::Array { values } => values
            .iter()
            .map(default_value)
            .collect::<Result<Vec<_>, _>>()
            .map(|values| DefaultValue::Array { values }),
        AttributeValue::Tuple { values } => values
            .iter()
            .map(default_value)
            .collect::<Result<Vec<_>, _>>()
            .map(|values| DefaultValue::Tuple { values }),
    }
}
