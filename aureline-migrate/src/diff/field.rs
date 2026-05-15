use std::collections::BTreeMap;

use aureline_core::schema_index::SchemaIndex;

use crate::change::{Change, FieldChangeSet};
use crate::diff::pair::{Diff, diff_by_key};

pub(crate) fn diff_table_fields(
    table_name: &str,
    prev: &SchemaIndex<'_>,
    new: &SchemaIndex<'_>,
    changes: &mut Vec<Change>,
) {
    let prev_fields = prev
        .fields_for_table(table_name)
        .collect::<BTreeMap<_, _>>();
    let new_fields = new.fields_for_table(table_name).collect::<BTreeMap<_, _>>();

    for (_name, change) in diff_by_key(&prev_fields, &new_fields) {
        match change {
            Diff::Added(field) => changes.push(Change::FieldAdded {
                table: table_name.to_string(),
                field: (*field).clone(),
            }),
            Diff::Removed(field) => changes.push(Change::FieldRemoved {
                table: table_name.to_string(),
                field: (*field).clone(),
            }),
            Diff::Change(prev, new) => {
                let field_changes = FieldChangeSet::between(prev, new);
                if !field_changes.is_empty() {
                    changes.push(Change::FieldChanged {
                        table: table_name.to_string(),
                        from: (*prev).clone(),
                        to: (*new).clone(),
                        changes: field_changes,
                    });
                }
            }
        }
    }
}
