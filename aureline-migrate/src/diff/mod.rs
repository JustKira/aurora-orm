use std::collections::HashMap;

use aureline_core::ast::{Schema, SchemaItem, Table};

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
    let mut changes = Vec::new();
    diff_analyzers(&prev, &new, &mut changes);
    diff_tables(&prev, &new, &mut changes);
    changes.retain(|change| !is_index_change(change));
    diff_indexes(&prev, &new, &mut changes);
    changes
}

pub fn diff_schemas(prev: &Schema, new: &Schema) -> Vec<Op> {
    plan_changes(diff_changes(prev, new)).steps
}

fn diff_indexes(prev: &Schema, new: &Schema, changes: &mut Vec<Change>) {
    let prev_tables = tables_by_name(prev);
    let new_tables = tables_by_name(new);

    for (name, change) in diff_by_key(&prev_tables, &new_tables) {
        match change {
            Diff::Added(table) => {
                let empty = table_without_indexes(table);
                diff_table_indexes(name, &empty, table, changes);
            }
            Diff::Removed(table) => {
                let empty = table_without_indexes(table);
                diff_table_indexes(name, table, &empty, changes);
            }
            Diff::Change(prev, new) => diff_table_indexes(name, prev, new, changes),
        }
    }
}

fn tables_by_name(schema: &Schema) -> HashMap<&str, &Table> {
    schema
        .items
        .iter()
        .filter_map(|item| match item {
            SchemaItem::TableDecl(table) => Some((table.name.as_str(), table)),
            SchemaItem::DocComment { .. } | SchemaItem::AnalyzerDecl(_) => None,
        })
        .collect()
}

fn table_without_indexes(table: &Table) -> Table {
    let mut table = table.clone();
    table.indexes.clear();
    table
}

fn is_index_change(change: &Change) -> bool {
    matches!(
        change,
        Change::IndexAdded { .. } | Change::IndexRemoved { .. } | Change::IndexChanged { .. }
    )
}
