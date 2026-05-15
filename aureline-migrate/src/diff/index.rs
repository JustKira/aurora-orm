use std::collections::BTreeMap;

use aureline_core::ast::{Index, IndexKind};
use aureline_core::schema_index::SchemaIndex;

use crate::change::Change;
use crate::diff::pair::{Diff, diff_by_key};

pub(crate) fn diff_table_indexes(
    table_name: &str,
    prev: &SchemaIndex<'_>,
    next: &SchemaIndex<'_>,
    changes: &mut Vec<Change>,
) {
    let prev_indexes = prev
        .indexes_for_table(table_name)
        .collect::<BTreeMap<_, _>>();
    let new_indexes = next
        .indexes_for_table(table_name)
        .collect::<BTreeMap<_, _>>();

    let diffs = diff_by_key(&prev_indexes, &new_indexes).collect::<Vec<_>>();

    for (_name, change) in &diffs {
        match change {
            Diff::Removed(index) => changes.push(Change::IndexRemoved {
                table: table_name.to_string(),
                index: (*index).clone(),
            }),
            Diff::Change(prev, new) => {
                if prev != new {
                    changes.push(Change::IndexChanged {
                        table: table_name.to_string(),
                        from: (*prev).clone(),
                        to: (*new).clone(),
                    });
                }
            }
            Diff::Added(_) => {}
        }
    }

    let mut additions = new_indexes
        .values()
        .copied()
        .filter(|index| !prev_indexes.contains_key(index.name.as_str()))
        .collect::<Vec<_>>();
    additions.sort_by(|left, right| compare_index_order(left, right));

    for index in additions {
        changes.push(Change::IndexAdded {
            table: table_name.to_string(),
            index: index.clone(),
        });
    }
}

fn compare_index_order(left: &Index, right: &Index) -> std::cmp::Ordering {
    match (hnsw_type_rank(left), hnsw_type_rank(right)) {
        (Some(left), Some(right)) => left.cmp(&right),
        _ => left.name.cmp(&right.name),
    }
}

fn hnsw_type_rank(index: &Index) -> Option<usize> {
    let IndexKind::Hnsw { ty: Some(ty), .. } = &index.kind else {
        return None;
    };

    match ty.as_str() {
        "f64" => Some(0),
        "f32" => Some(1),
        "i64" => Some(2),
        "i32" => Some(3),
        "i16" => Some(4),
        _ => None,
    }
}
