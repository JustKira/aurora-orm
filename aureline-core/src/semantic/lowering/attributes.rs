use crate::ast::{Attribute, AttributeValue, Field, Schema, SchemaItem, Table, Type};

use super::super::diagnostics::unknown_attribute;
use super::super::{AttributeScope, SemanticError, SemanticResult};
use super::{assertions, flexible, fulltext, hnsw, indexes, permissions};

/// Lower raw `@`/`@@` attributes into structured schema fields.
///
/// The parser stores attributes as generic syntax. This pass gives them
/// meaning by populating `Table.indexes` and `Field.flexible`, while preserving
/// the raw attributes for tooling.
pub fn lower(schema: &mut Schema) -> SemanticResult {
    let mut errors = Vec::new();

    for item in &mut schema.items {
        if let SchemaItem::TableDecl(table) = item {
            lower_table(table, &mut errors);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn lower_table(table: &mut Table, errors: &mut Vec<SemanticError>) {
    let table_name = table.name.clone();
    let mut lowered_indexes = Vec::new();

    for field in &mut table.fields {
        lower_field_attributes(&table_name, field, &mut lowered_indexes, errors);
    }

    for attr in &table.raw_attributes {
        lower_block_attribute(
            &table_name,
            &table.fields,
            attr,
            &mut lowered_indexes,
            errors,
        );
    }

    indexes::validate_names(&table_name, &lowered_indexes, errors);
    table.indexes = lowered_indexes
        .into_iter()
        .map(|lowered| lowered.index)
        .collect();
}

const FIELD_ATTRS: &[&str] = &[
    "unique", "index", "flexible", "hnsw", "fulltext", "assert", "allow",
];

fn lower_field_attributes(
    table: &str,
    field: &mut Field,
    lowered_indexes: &mut Vec<indexes::LoweredIndex>,
    errors: &mut Vec<SemanticError>,
) {
    for attr in &field.raw_attributes {
        match attr.name.as_str() {
            "unique" => indexes::lower_field_unique(table, field, attr, lowered_indexes, errors),
            "index" => indexes::lower_field_index(table, field, attr, lowered_indexes, errors),
            "flexible" => {
                if flexible::lower(table, &field.name, &field.ty, attr, errors) {
                    field.flexible = true;
                }
            }
            "hnsw" => hnsw::lower(table, field, attr, lowered_indexes, errors),
            "fulltext" => fulltext::lower(table, field, attr, lowered_indexes, errors),
            "assert" => assertions::lower(attr, errors),
            "allow" => permissions::lower(attr, errors),
            unknown => errors.push(at_attr(
                unknown_attribute(AttributeScope::Field, unknown, FIELD_ATTRS),
                attr,
            )),
        }
    }
}

// Block-level annotations are only for *composite* or *table-level* concepts.
// Single-field indexes (fulltext, hnsw, plain index, unique) live as field-level
// `@`-annotations only. We deliberately don't accept `@@fulltext` / `@@hnsw`
// because they'd duplicate the field-level forms and invite drift.
const BLOCK_ATTRS: &[&str] = &["index", "unique", "count"];

fn lower_block_attribute(
    table: &str,
    fields: &[Field],
    attr: &Attribute,
    lowered_indexes: &mut Vec<indexes::LoweredIndex>,
    errors: &mut Vec<SemanticError>,
) {
    match attr.name.as_str() {
        "count" => indexes::lower_block_count(table, attr, lowered_indexes, errors),
        "index" => indexes::lower_block_index(table, fields, attr, lowered_indexes, errors),
        "unique" => indexes::lower_block_unique(table, fields, attr, lowered_indexes, errors),
        unknown => errors.push(at_attr(
            unknown_attribute(AttributeScope::Block, unknown, BLOCK_ATTRS),
            attr,
        )),
    }
}

pub(super) fn err(message: String) -> SemanticError {
    SemanticError {
        message,
        hint: None,
        range: None,
    }
}

pub(super) fn err_at(attr: &Attribute, message: String) -> SemanticError {
    at_attr(err(message), attr)
}

pub(super) fn at_attr(mut error: SemanticError, attr: &Attribute) -> SemanticError {
    error.range = attr.source_range;
    error
}

pub(super) fn ident_value(v: &AttributeValue, label: &str) -> Result<String, SemanticError> {
    match v {
        AttributeValue::Ident { value } => Ok(value.clone()),
        _ => Err(err(format!("{label}: expected an identifier"))),
    }
}

/// Accept either an identifier or a string literal - used for places like
/// `name:` where users might write `name: foo` or `name: "foo"`.
pub(super) fn name_value(v: &AttributeValue, label: &str) -> Result<String, SemanticError> {
    match v {
        AttributeValue::Ident { value } | AttributeValue::String { value } => Ok(value.clone()),
        _ => Err(err(format!("{label}: expected an identifier or string"))),
    }
}

pub(super) fn uint_value(v: &AttributeValue, label: &str) -> Result<u32, SemanticError> {
    match v {
        AttributeValue::Number { value } => {
            if value.fract() != 0.0 || *value < 0.0 {
                return Err(err(format!(
                    "{label}: expected a non-negative integer, got {value}"
                )));
            }
            Ok(*value as u32)
        }
        _ => Err(err(format!("{label}: expected a non-negative integer"))),
    }
}

pub(super) fn uint_value_bounded(
    v: &AttributeValue,
    label: &str,
    max: u32,
) -> Result<u32, SemanticError> {
    let value = uint_value(v, label)?;
    if value > max {
        return Err(err(format!(
            "{label}: expected a value <= {max}, got {value}"
        )));
    }
    Ok(value)
}

pub(super) fn number_value(v: &AttributeValue, label: &str) -> Result<f64, SemanticError> {
    match v {
        AttributeValue::Number { value } => Ok(*value),
        _ => Err(err(format!("{label}: expected a number"))),
    }
}

pub(super) fn is_object(t: &Type) -> bool {
    matches!(t, Type::Primitive { name } if name == "object")
}

pub(super) fn is_string(t: &Type) -> bool {
    matches!(t, Type::Primitive { name } if name == "string")
}

pub(super) fn is_array_of_float(t: &Type) -> bool {
    matches!(t, Type::Array { inner, .. } if matches!(inner.as_ref(), Type::Primitive { name } if name == "float"))
}

pub(super) fn type_label(t: &Type) -> String {
    crate::emit::surql_type(t)
}

pub(super) fn auto_name(table: &str, fields: &[String], suffix: &str) -> String {
    let table_part = crate::emit::pascal_to_snake(table);
    if fields.is_empty() {
        format!("{table_part}_{suffix}")
    } else {
        format!("{table_part}_{}_{suffix}", fields.join("_"))
    }
}
