use aureline_core::schema_index::SchemaIndex;

use crate::change::Change;
use crate::diff::pair::{Diff, diff_by_key};

pub fn diff_analyzers(prev: &SchemaIndex<'_>, next: &SchemaIndex<'_>, changes: &mut Vec<Change>) {
    for (_name, change) in diff_by_key(&prev.analyzers, &next.analyzers) {
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
