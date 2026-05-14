use aureline_core::ast::Schema;

use crate::change::Change;
use crate::ops::Op;
use crate::plan::plan_changes;
use crate::schema::table_field_schema;

mod field;
mod pair;
mod table;

pub fn diff_changes(prev: &Schema, new: &Schema) -> Vec<Change> {
    let prev = table_field_schema(prev);
    let new = table_field_schema(new);
    let mut changes = Vec::new();
    table::diff_tables(&prev, &new, &mut changes);
    changes
}

pub fn diff_schemas(prev: &Schema, new: &Schema) -> Vec<Op> {
    plan_changes(diff_changes(prev, new)).steps
}
