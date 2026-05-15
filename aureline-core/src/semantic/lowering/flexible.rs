use crate::ast::{Attribute, Type};

use super::super::SemanticError;
use super::attributes::{err_at, is_object, type_label};

pub(super) fn lower(
    table: &str,
    field_name: &str,
    field_type: &Type,
    attr: &Attribute,
    errors: &mut Vec<SemanticError>,
) -> bool {
    if !attr.args.is_empty() {
        errors.push(err_at(
            attr,
            format!(
                "@flexible on {table}.{field} takes no arguments",
                field = field_name
            ),
        ));
        return false;
    }
    if !is_object(field_type) {
        errors.push(err_at(
            attr,
            format!(
                "@flexible on {table}.{field} requires `object`, got `{ty}`",
                field = field_name,
                ty = type_label(field_type),
            ),
        ));
        return false;
    }
    true
}
