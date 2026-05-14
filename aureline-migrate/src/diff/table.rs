use std::collections::HashMap;

use aureline_core::ast::{Schema, SchemaItem, Table};

use crate::change::Change;
use crate::diff::field::diff_table_fields;
use crate::diff::pair::{Diff, diff_by_key};

pub(crate) fn diff_tables(prev: &Schema, new: &Schema, changes: &mut Vec<Change>) {
    let prev_tables = tables_by_name(prev);
    let new_tables = tables_by_name(new);
    for (name, change) in diff_by_key(&prev_tables, &new_tables) {
        match change {
            Diff::Added(table) => {
                changes.push(Change::TableAdded((*table).clone()));
            }
            Diff::Removed => changes.push(Change::TableRemoved(name.to_string())),
            Diff::Change(prev, new) => {
                if prev.modifier != new.modifier {
                    changes.push(Change::TableModeChanged {
                        table: name.to_string(),
                        from: prev.modifier.clone(),
                        to: new.modifier.clone(),
                    });
                }
                diff_table_members(name, prev, new, changes);
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

fn diff_table_members(table_name: &str, prev: &Table, new: &Table, changes: &mut Vec<Change>) {
    diff_table_fields(table_name, prev, new, changes);
}
