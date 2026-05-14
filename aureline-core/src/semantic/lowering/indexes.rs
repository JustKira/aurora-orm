use std::collections::HashSet;

use crate::ast::{Attribute, AttributeArg, AttributeValue, Field, Index, IndexKind};

use super::super::SemanticError;
use super::attributes::{at_attr, auto_name, err, err_at, name_value};

pub(super) fn lower_field_unique(
    table: &str,
    field: &Field,
    attr: &Attribute,
    indexes: &mut Vec<Index>,
    errors: &mut Vec<SemanticError>,
) {
    match parse_named_only(&attr.args, "@unique").map_err(|error| at_attr(error, attr)) {
        Ok(name_override) => indexes.push(Index {
            name: name_override
                .unwrap_or_else(|| auto_name(table, std::slice::from_ref(&field.name), "unique")),
            fields: vec![field.name.clone()],
            kind: IndexKind::Unique,
        }),
        Err(error) => errors.push(error),
    }
}

pub(super) fn lower_field_index(
    table: &str,
    field: &Field,
    attr: &Attribute,
    indexes: &mut Vec<Index>,
    errors: &mut Vec<SemanticError>,
) {
    match parse_named_only(&attr.args, "@index").map_err(|error| at_attr(error, attr)) {
        Ok(name_override) => indexes.push(Index {
            name: name_override
                .unwrap_or_else(|| auto_name(table, std::slice::from_ref(&field.name), "idx")),
            fields: vec![field.name.clone()],
            kind: IndexKind::Standard,
        }),
        Err(error) => errors.push(error),
    }
}

pub(super) fn lower_block_count(
    table: &str,
    attr: &Attribute,
    indexes: &mut Vec<Index>,
    errors: &mut Vec<SemanticError>,
) {
    if !attr.args.is_empty() {
        errors.push(err_at(
            attr,
            format!("@@count on {table} takes no arguments"),
        ));
        return;
    }
    indexes.push(Index {
        name: auto_name(table, &[], "count"),
        fields: Vec::new(),
        kind: IndexKind::Count,
    });
}

pub(super) fn lower_block_index(
    table: &str,
    fields: &[Field],
    attr: &Attribute,
    indexes: &mut Vec<Index>,
    errors: &mut Vec<SemanticError>,
) {
    match parse_field_list_block(table, fields, &attr.args, "@@index")
        .map_err(|error| at_attr(error, attr))
    {
        Ok((field_names, name_override)) => {
            indexes.push(Index {
                name: name_override.unwrap_or_else(|| auto_name(table, &field_names, "idx")),
                fields: field_names,
                kind: IndexKind::Standard,
            });
        }
        Err(error) => errors.push(error),
    }
}

pub(super) fn lower_block_unique(
    table: &str,
    fields: &[Field],
    attr: &Attribute,
    indexes: &mut Vec<Index>,
    errors: &mut Vec<SemanticError>,
) {
    match parse_field_list_block(table, fields, &attr.args, "@@unique")
        .map_err(|error| at_attr(error, attr))
    {
        Ok((field_names, name_override)) => {
            indexes.push(Index {
                name: name_override.unwrap_or_else(|| auto_name(table, &field_names, "unique")),
                fields: field_names,
                kind: IndexKind::Unique,
            });
        }
        Err(error) => errors.push(error),
    }
}

pub(super) fn validate_names(table: &str, indexes: &[Index], errors: &mut Vec<SemanticError>) {
    let mut seen = HashSet::new();
    for index in indexes {
        if !seen.insert(index.name.as_str()) {
            errors.push(err(format!(
                "duplicate index name `{}` on table {table}",
                index.name
            )));
        }
    }
}

/// `@unique` / `@index` accept only one optional kw arg: `name: "..."`.
fn parse_named_only(args: &[AttributeArg], label: &str) -> Result<Option<String>, SemanticError> {
    let mut name = None;
    for arg in args {
        match arg {
            AttributeArg::Keyword { name: key, value } => match key.as_str() {
                "name" => name = Some(name_value(value, &format!("{label} name"))?),
                other => {
                    return Err(err(format!(
                        "unknown {label} arg `{other}`; expected `name`"
                    )));
                }
            },
            AttributeArg::Positional { .. } => {
                return Err(err(format!("{label} does not accept positional arguments")));
            }
        }
    }
    Ok(name)
}

fn parse_field_list_block(
    table: &str,
    fields: &[Field],
    args: &[AttributeArg],
    label: &str,
) -> Result<(Vec<String>, Option<String>), SemanticError> {
    let mut field_names: Option<Vec<String>> = None;
    let mut name: Option<String> = None;
    for arg in args {
        let AttributeArg::Keyword { name: key, value } = arg else {
            return Err(err(format!("{label} does not accept positional arguments")));
        };
        match key.as_str() {
            "fields" => {
                let AttributeValue::Array { values } = value else {
                    return Err(err(format!(
                        "{label} on {table}: `fields:` expects an array of identifiers"
                    )));
                };
                let mut names = Vec::with_capacity(values.len());
                for value in values {
                    match value {
                        AttributeValue::Ident { value } => names.push(value.clone()),
                        _ => {
                            return Err(err(format!(
                                "{label} on {table}: `fields:` array must contain identifiers"
                            )));
                        }
                    }
                }
                field_names = Some(names);
            }
            "name" => name = Some(name_value(value, &format!("{label} name"))?),
            other => {
                return Err(err(format!(
                    "unknown {label} arg `{other}`; expected one of: fields, name"
                )));
            }
        }
    }
    let field_names = field_names.ok_or_else(|| {
        err(format!(
            "{label} on {table} requires a `fields: [...]` argument"
        ))
    })?;
    if field_names.is_empty() {
        return Err(err(format!(
            "{label} on {table} requires at least one field"
        )));
    }
    let mut seen = HashSet::new();
    for field_name in &field_names {
        if !seen.insert(field_name.as_str()) {
            return Err(err(format!(
                "{label} on {table}: duplicate field `{field_name}`"
            )));
        }
        if !fields.iter().any(|field| &field.name == field_name) {
            return Err(err(format!(
                "{label} on {table}: unknown field `{field_name}`"
            )));
        }
    }
    Ok((field_names, name))
}
