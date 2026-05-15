use aureline_core::ast::Schema;
use aureline_core::schema_index::SchemaIndex;

use crate::change::Change;
use crate::diff::pair::{Diff, diff_by_key};
use crate::diff::table::diff_tables;
use crate::ops::Op;
use crate::plan::plan_changes;
use crate::schema::full_schema;

pub mod analyzer;
pub mod field;
pub mod index;
pub mod pair;
pub mod table;

pub use analyzer::diff_analyzers;

use index::diff_table_indexes;

pub fn diff_changes(prev: &Schema, new: &Schema) -> Vec<Change> {
    let prev = full_schema(prev);
    let new = full_schema(new);
    let prev_index = SchemaIndex::from_schema(&prev);
    let new_index = SchemaIndex::from_schema(&new);
    let mut changes = Vec::new();
    diff_analyzers(&prev_index, &new_index, &mut changes);
    diff_tables(&prev_index, &new_index, &mut changes);
    changes.retain(|change| !is_index_change(change));
    diff_indexes(&prev_index, &new_index, &mut changes);
    changes
}

pub fn diff_schemas(prev: &Schema, new: &Schema) -> Vec<Op> {
    plan_changes(diff_changes(prev, new)).steps
}

fn diff_indexes(prev: &SchemaIndex<'_>, new: &SchemaIndex<'_>, changes: &mut Vec<Change>) {
    for (name, change) in diff_by_key(&prev.tables, &new.tables) {
        match change {
            Diff::Added(_) | Diff::Change(_, _) => {
                diff_table_indexes(name, prev, new, changes);
            }
            Diff::Removed(_) => {}
        }
    }
}

fn is_index_change(change: &Change) -> bool {
    matches!(
        change,
        Change::IndexAdded { .. } | Change::IndexRemoved { .. } | Change::IndexChanged { .. }
    )
}
