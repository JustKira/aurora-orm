use std::collections::HashSet;

use crate::ast::{Schema, SchemaItem};

#[derive(Debug)]
pub(super) struct AnalysisContext {
    // Raw user-facing names as written in Aureline source.
    table_names: HashSet<String>,
    // Emitted SurrealDB names. This catches `User` and `user` colliding after
    // `pascal_to_snake` normalization.
    normalized_table_names: HashSet<String>,
    // Analyzer declarations are referenced by full-text indexes.
    analyzer_names: HashSet<String>,
}

impl AnalysisContext {
    pub(super) fn new(schema: &Schema) -> Self {
        let mut table_names = HashSet::new();
        let mut normalized_table_names = HashSet::new();
        let mut analyzer_names = HashSet::new();

        for item in &schema.items {
            match item {
                SchemaItem::TableDecl(table) => {
                    table_names.insert(table.name.clone());
                    normalized_table_names.insert(normalized_name(&table.name));
                }
                SchemaItem::AnalyzerDecl(analyzer) => {
                    analyzer_names.insert(analyzer.name.clone());
                }
                SchemaItem::DocComment { .. } => {}
            }
        }

        Self {
            table_names,
            normalized_table_names,
            analyzer_names,
        }
    }

    pub(super) fn has_table(&self, name: &str) -> bool {
        // Record references should work whether the user writes the declared
        // table name (`User`) or the normalized SurrealDB table name (`user`).
        self.table_names.contains(name)
            || self.normalized_table_names.contains(&normalized_name(name))
    }

    pub(super) fn has_analyzer(&self, name: &str) -> bool {
        self.analyzer_names.contains(name)
    }
}

pub(super) fn normalized_name(name: &str) -> String {
    // Reuse the emitter's naming rule so semantic collision checks match the
    // SQL we will actually generate.
    crate::emit::pascal_to_snake(name)
}
