use aureline_core::schema_index::SchemaIndex;

use crate::change::Change;
use crate::diff::field::diff_table_fields;
use crate::diff::index::diff_table_indexes;
use crate::diff::pair::{Diff, diff_by_key};

pub(crate) fn diff_tables(
    prev_index: &SchemaIndex<'_>,
    new_index: &SchemaIndex<'_>,
    changes: &mut Vec<Change>,
) {
    for (name, change) in diff_by_key(&prev_index.tables, &new_index.tables) {
        match change {
            Diff::Added(table) => {
                changes.push(Change::TableAdded((*table).clone()));
            }
            Diff::Removed(_) => changes.push(Change::TableRemoved(name.to_string())),
            Diff::Change(prev_table, new_table) => {
                if prev_table.modifier != new_table.modifier {
                    changes.push(Change::TableModeChanged {
                        table: name.to_string(),
                        from: prev_table.modifier.clone(),
                        to: new_table.modifier.clone(),
                    });
                }
                diff_table_members(name, prev_index, new_index, changes);
            }
        }
    }
}

fn diff_table_members(
    table_name: &str,
    prev: &SchemaIndex<'_>,
    new: &SchemaIndex<'_>,
    changes: &mut Vec<Change>,
) {
    diff_table_fields(table_name, prev, new, changes);
    diff_table_indexes(table_name, prev, new, changes);
}
