use std::collections::HashMap;

use aurora_core::ast::{Field, Table};

use crate::diff::pair::{Diff, diff_by_key};
use crate::ops::Op;

pub(crate) fn diff_table_fields(table_name: &str, prev: &Table, new: &Table, ops: &mut Vec<Op>) {
    let prev_fields = fields_by_name(&prev.fields);
    let new_fields = fields_by_name(&new.fields);

    for (name, change) in diff_by_key(&prev_fields, &new_fields) {
        match change {
            Diff::Added(field) => ops.push(Op::AddField {
                table: table_name.to_string(),
                field: (*field).clone(),
            }),
            Diff::Removed => ops.push(Op::RemoveField {
                table: table_name.to_string(),
                field: name.to_string(),
            }),
            // Type/optional/flexible changes — guard arms are mutually exclusive.
            // Type change re-emits the field with its current optional/flexible.
            Diff::Change(prev, new) if prev.ty != new.ty => {
                ops.push(Op::ChangeFieldType {
                    table: table_name.to_string(),
                    field: (*new).clone(),
                    from_type: prev.ty.clone(),
                });
            }
            Diff::Change(prev, new) if prev.optional != new.optional => {
                ops.push(Op::ChangeFieldOptional {
                    table: table_name.to_string(),
                    field: (*new).clone(),
                    now_optional: new.optional,
                });
            }
            Diff::Change(prev, new) if prev.flexible != new.flexible => {
                ops.push(Op::ChangeFieldFlexible {
                    table: table_name.to_string(),
                    field: (*new).clone(),
                    now_flexible: new.flexible,
                });
            }
            Diff::Change(_, _) => {}
        }
    }
}

fn fields_by_name(fields: &[Field]) -> HashMap<&str, &Field> {
    fields
        .iter()
        .map(|field| (field.name.as_str(), field))
        .collect()
}
