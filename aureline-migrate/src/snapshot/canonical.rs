use aureline_core::ast::{Analyzer, Field, Index, Schema, SchemaItem, Table};
use serde_json::{Value, json};

use crate::schema::table_field_schema;

use super::SNAPSHOT_VERSION;

pub fn canonicalize(schema: &Schema) -> String {
    let schema = table_field_schema(schema);
    let tables: Vec<Value> = sorted_tables(&schema)
        .iter()
        .map(|table| canonicalize_table(table))
        .collect();

    let analyzers: Vec<Value> = sorted_analyzers(&schema)
        .iter()
        .map(|a| canonicalize_analyzer(a))
        .collect();

    let root = json!({
        "version": SNAPSHOT_VERSION,
        "analyzers": analyzers,
        "tables": tables,
    });
    serde_json::to_string_pretty(&root).expect("canonical snapshot is valid JSON")
}

/// Returns the schema's tables in alphabetical order, dropping non-table items.
fn sorted_tables(schema: &Schema) -> Vec<&Table> {
    let mut tables: Vec<&Table> = schema
        .items
        .iter()
        .filter_map(|item| match item {
            SchemaItem::TableDecl(table) => Some(table),
            SchemaItem::DocComment { .. } | SchemaItem::AnalyzerDecl(_) => None,
        })
        .collect();
    tables.sort_by(|a, b| a.name.cmp(&b.name));
    tables
}

fn sorted_analyzers(schema: &Schema) -> Vec<&Analyzer> {
    let mut analyzers: Vec<&Analyzer> = schema
        .items
        .iter()
        .filter_map(|item| match item {
            SchemaItem::AnalyzerDecl(a) => Some(a),
            SchemaItem::DocComment { .. } | SchemaItem::TableDecl(_) => None,
        })
        .collect();
    analyzers.sort_by(|a, b| a.name.cmp(&b.name));
    analyzers
}

fn canonicalize_table(table: &Table) -> Value {
    let mut fields: Vec<&Field> = table.fields.iter().collect();
    fields.sort_by(|a, b| a.name.cmp(&b.name));
    let fields: Vec<Value> = fields.iter().map(|f| canonicalize_field(f)).collect();

    let mut indexes: Vec<&Index> = table.indexes.iter().collect();
    indexes.sort_by(|a, b| a.name.cmp(&b.name));
    let indexes: Vec<Value> = indexes.iter().map(|i| canonicalize_index(i)).collect();

    json!({
        "name": table.name,
        "modifier": table.modifier,
        "fields": fields,
        "indexes": indexes,
    })
}

fn canonicalize_field(field: &Field) -> Value {
    json!({
        "name": field.name,
        "type": field.ty,
        "optional": field.optional,
        "flexible": field.flexible,
    })
}

fn canonicalize_index(index: &Index) -> Value {
    json!({
        "name": index.name,
        "fields": index.fields,
        "kind": index.kind,
    })
}

fn canonicalize_analyzer(a: &Analyzer) -> Value {
    json!({
        "name": a.name,
        "tokenizers": a.tokenizers,
        "filters": a.filters,
    })
}
