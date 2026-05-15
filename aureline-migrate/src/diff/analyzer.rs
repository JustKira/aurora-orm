use std::collections::HashMap;

use aureline_core::ast::{Analyzer, Schema, SchemaItem};

use crate::change::Change;
use crate::diff::pair::{Diff, diff_by_key};

pub fn diff_analyzers(prev: &Schema, next: &Schema, changes: &mut Vec<Change>) {
    let prev_analyzers = analyzers_by_name(prev);
    let new_analyzers = analyzers_by_name(next);

    for (_name, change) in diff_by_key(&prev_analyzers, &new_analyzers) {
        match change {
            Diff::Added(analyzer) => {
                changes.push(Change::AnalyzerAdded((*analyzer).clone()));
            }
            Diff::Removed(analyzer) => {
                changes.push(Change::AnalyzerRemoved((*analyzer).clone()));
            }
            Diff::Change(prev, new) => {
                if prev != new {
                    changes.push(Change::AnalyzerChanged {
                        from: (*prev).clone(),
                        to: (*new).clone(),
                    });
                }
            }
        }
    }
}

fn analyzers_by_name(schema: &Schema) -> HashMap<&str, &Analyzer> {
    schema
        .items
        .iter()
        .filter_map(|item| match item {
            SchemaItem::AnalyzerDecl(analyzer) => Some((analyzer.name.as_str(), analyzer)),
            SchemaItem::DocComment { .. } | SchemaItem::TableDecl(_) => None,
        })
        .collect()
}
