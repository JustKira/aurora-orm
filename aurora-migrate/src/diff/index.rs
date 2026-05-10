use std::collections::HashMap;

use aurora_core::ast::{Index, Table};

use crate::diff::pair::{Diff, diff_by_key};
use crate::ops::Op;

pub(crate) fn diff_table_indexes(table_name: &str, prev: &Table, new: &Table, ops: &mut Vec<Op>) {
    let prev_idx = indexes_by_name(&prev.indexes);
    let new_idx = indexes_by_name(&new.indexes);

    for (name, change) in diff_by_key(&prev_idx, &new_idx) {
        match change {
            Diff::Added(index) => ops.push(Op::CreateIndex {
                table: table_name.to_string(),
                index: (*index).clone(),
            }),
            Diff::Removed => ops.push(Op::RemoveIndex {
                table: table_name.to_string(),
                name: name.to_string(),
            }),
            Diff::Change(prev_i, new_i) if prev_i != new_i => ops.push(Op::ChangeIndex {
                table: table_name.to_string(),
                name: name.to_string(),
                from: (*prev_i).clone(),
                to: (*new_i).clone(),
            }),
            Diff::Change(_, _) => {}
        }
    }
}

fn indexes_by_name(indexes: &[Index]) -> HashMap<&str, &Index> {
    indexes.iter().map(|i| (i.name.as_str(), i)).collect()
}
