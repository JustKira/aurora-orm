use std::collections::HashMap;

use aureline_core::ast::{Analyzer, Schema};
use aureline_core::schema_index::SchemaIndex;

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
    let index = SchemaIndex::from_schema(schema);
    index
        .analyzers
        .iter()
        .map(|(&name, &analyzer)| (name, analyzer))
        .collect()
}
