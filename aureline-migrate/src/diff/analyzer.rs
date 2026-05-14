use std::collections::HashMap;

use aureline_core::ast::{Analyzer, Schema, SchemaItem};

use crate::diff::pair::{Diff, diff_by_key};
use crate::ops::Op;

pub(crate) fn diff_analyzers(prev: &Schema, new: &Schema, ops: &mut Vec<Op>) {
    let prev_analyzers = analyzers_by_name(prev);
    let new_analyzers = analyzers_by_name(new);
    for (name, change) in diff_by_key(&prev_analyzers, &new_analyzers) {
        match change {
            Diff::Added(a) => ops.push(Op::CreateAnalyzer((*a).clone())),
            Diff::Removed => ops.push(Op::RemoveAnalyzer(name.to_string())),
            Diff::Change(prev_a, new_a) if prev_a != new_a => {
                ops.push(Op::ChangeAnalyzer {
                    from: (*prev_a).clone(),
                    to: (*new_a).clone(),
                });
            }
            Diff::Change(_, _) => {}
        }
    }
}

fn analyzers_by_name(schema: &Schema) -> HashMap<&str, &Analyzer> {
    schema
        .items
        .iter()
        .filter_map(|item| match item {
            SchemaItem::AnalyzerDecl(a) => Some((a.name.as_str(), a)),
            SchemaItem::DocComment { .. }
            | SchemaItem::SurqlBlock(_)
            | SchemaItem::TableDecl(_) => None,
        })
        .collect()
}
