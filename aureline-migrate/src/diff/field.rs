use std::collections::HashMap;

use aureline_core::ast::{Field, Table};

use crate::change::{Change, FieldChangeSet};
use crate::diff::pair::{Diff, diff_by_key};

pub(crate) fn diff_table_fields(
    table_name: &str,
    prev: &Table,
    new: &Table,
    changes: &mut Vec<Change>,
) {
    let prev_fields = fields_by_name(&prev.fields);
    let new_fields = fields_by_name(&new.fields);

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

fn fields_by_name(fields: &[Field]) -> HashMap<&str, &Field> {
    fields
        .iter()
        .map(|field| (field.name.as_str(), field))
        .collect()
}
