use std::collections::HashMap;

use aurora_core::ast::{Schema, SchemaItem, Table};

use crate::diff::field::diff_table_fields;
use crate::diff::index::diff_table_indexes;
use crate::diff::pair::{Diff, diff_by_key};
use crate::ops::Op;

pub(crate) fn diff_tables(prev: &Schema, new: &Schema, ops: &mut Vec<Op>) {
    let prev_tables = tables_by_name(prev);
    let new_tables = tables_by_name(new);
    for (name, change) in diff_by_key(&prev_tables, &new_tables) {
        match change {
            Diff::Added(table) => {
                ops.push(Op::CreateTable((*table).clone()));
                // Indexes on a freshly-created table — emit them after CreateTable.
                for idx in &table.indexes {
                    ops.push(Op::CreateIndex {
                        table: name.to_string(),
                        index: idx.clone(),
                    });
                }
            }
            Diff::Removed => ops.push(Op::RemoveTable(name.to_string())),
            Diff::Change(prev, new) => {
                if prev.modifier != new.modifier {
                    ops.push(Op::ChangeTableMode {
                        table: name.to_string(),
                        from: prev.modifier.clone(),
                        to: new.modifier.clone(),
                    });
                }
                diff_table_members(name, prev, new, ops);
            }
        }
    }
}

fn tables_by_name(schema: &Schema) -> HashMap<&str, &Table> {
    schema
        .items
        .iter()
        .filter_map(|item| match item {
            SchemaItem::TableDecl(table) => Some((table.name.as_str(), table)),
            SchemaItem::DocComment { .. }
            | SchemaItem::SurqlBlock(_)
            | SchemaItem::AnalyzerDecl(_) => None,
        })
        .collect()
}

fn diff_table_members(table_name: &str, prev: &Table, new: &Table, ops: &mut Vec<Op>) {
    diff_table_fields(table_name, prev, new, ops);
    diff_table_indexes(table_name, prev, new, ops);
}
