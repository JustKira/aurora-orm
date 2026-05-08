use std::collections::{BTreeSet, HashMap};

use aurora_core::ast::{Analyzer, Field, Index, Schema, SchemaItem, Table};

use crate::ops::Op;

/// Where a key sits relative to two schemas being compared. Lets callers
/// match on `Added` / `Removed` / `Both` instead of decoding `Option`
/// tuples like `(None, Some)`.
enum Diff<'a, T> {
    /// Key is only on the new side — the item appeared.
    Added(&'a T),
    /// Key is only on the prev side — the item disappeared. No payload because
    /// callers only need the key name (from the iterator tuple) to emit the op.
    Removed,
    /// Key exists on both sides. Whether it actually *changed* is up to the
    /// caller to decide by comparing `(prev, new)`.
    Both(&'a T, &'a T),
}

/// Yields `(key, change)` for every key present in either map, sorted so
/// the resulting op order is deterministic. Used at every diff level
/// (tables, fields, indexes, analyzers).
fn diff_by_key<'a, K, V>(
    prev: &'a HashMap<K, &'a V>,
    new: &'a HashMap<K, &'a V>,
) -> impl Iterator<Item = (K, Diff<'a, V>)> + 'a
where
    K: Ord + Copy + Eq + std::hash::Hash,
{
    let keys: BTreeSet<K> = prev.keys().chain(new.keys()).copied().collect();
    keys.into_iter().map(move |k| {
        let change = match (prev.get(&k), new.get(&k)) {
            (None, Some(&n)) => Diff::Added(n),
            (Some(_), None) => Diff::Removed,
            (Some(&p), Some(&n)) => Diff::Both(p, n),
            // Impossible: `k` was drawn from the union of both maps' keys.
            (None, None) => unreachable!("key came from union of both maps"),
        };
        (k, change)
    })
}

pub fn diff_schemas(prev: &Schema, new: &Schema) -> Vec<Op> {
    let mut ops = Vec::new();

    // Analyzers (top-level).
    let prev_analyzers = analyzers_by_name(prev);
    let new_analyzers = analyzers_by_name(new);
    for (name, change) in diff_by_key(&prev_analyzers, &new_analyzers) {
        match change {
            Diff::Added(a) => ops.push(Op::CreateAnalyzer((*a).clone())),
            Diff::Removed => ops.push(Op::RemoveAnalyzer(name.to_string())),
            Diff::Both(prev_a, new_a) if prev_a != new_a => {
                ops.push(Op::ChangeAnalyzer {
                    from: (*prev_a).clone(),
                    to: (*new_a).clone(),
                });
            }
            Diff::Both(_, _) => {}
        }
    }

    // Tables.
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
            Diff::Both(prev, new) => {
                if prev.modifier != new.modifier {
                    ops.push(Op::ChangeTableMode {
                        table: name.to_string(),
                        from: prev.modifier.clone(),
                        to: new.modifier.clone(),
                    });
                }
                diff_table_members(name, prev, new, &mut ops);
            }
        }
    }

    ops
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

fn analyzers_by_name(schema: &Schema) -> HashMap<&str, &Analyzer> {
    schema
        .items
        .iter()
        .filter_map(|item| match item {
            SchemaItem::AnalyzerDecl(a) => Some((a.name.as_str(), a)),
            SchemaItem::DocComment { .. } | SchemaItem::TableDecl(_) => None,
        })
        .collect()
}

fn diff_table_members(table_name: &str, prev: &Table, new: &Table, ops: &mut Vec<Op>) {
    diff_table_fields(table_name, prev, new, ops);
    diff_table_indexes(table_name, prev, new, ops);
}

fn diff_table_fields(table_name: &str, prev: &Table, new: &Table, ops: &mut Vec<Op>) {
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
            Diff::Both(prev, new) if prev.ty != new.ty => {
                ops.push(Op::ChangeFieldType {
                    table: table_name.to_string(),
                    field: (*new).clone(),
                    from_type: prev.ty.clone(),
                });
            }
            Diff::Both(prev, new) if prev.optional != new.optional => {
                ops.push(Op::ChangeFieldOptional {
                    table: table_name.to_string(),
                    field: (*new).clone(),
                    now_optional: new.optional,
                });
            }
            Diff::Both(prev, new) if prev.flexible != new.flexible => {
                ops.push(Op::ChangeFieldFlexible {
                    table: table_name.to_string(),
                    field: (*new).clone(),
                    now_flexible: new.flexible,
                });
            }
            Diff::Both(_, _) => {}
        }
    }
}

fn diff_table_indexes(table_name: &str, prev: &Table, new: &Table, ops: &mut Vec<Op>) {
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
            Diff::Both(prev_i, new_i) if prev_i != new_i => ops.push(Op::ChangeIndex {
                table: table_name.to_string(),
                name: name.to_string(),
                from: (*prev_i).clone(),
                to: (*new_i).clone(),
            }),
            Diff::Both(_, _) => {}
        }
    }
}

fn fields_by_name(fields: &[Field]) -> HashMap<&str, &Field> {
    fields
        .iter()
        .map(|field| (field.name.as_str(), field))
        .collect()
}

fn indexes_by_name(indexes: &[Index]) -> HashMap<&str, &Index> {
    indexes.iter().map(|i| (i.name.as_str(), i)).collect()
}
