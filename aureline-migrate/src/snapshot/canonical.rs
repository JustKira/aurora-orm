use aureline_core::ast::{Analyzer, Field, Index, Schema, Table};
use aureline_core::schema_index::SchemaIndex;
use serde_json::{Value, json};

use crate::schema::full_schema;

use super::SNAPSHOT_VERSION;

pub fn canonicalize(schema: &Schema) -> String {
    let schema = full_schema(schema);
    let index = SchemaIndex::from_schema(&schema);

    let tables: Vec<Value> = index
        .tables()
        .map(|(_, table)| canonicalize_table(&index, table))
        .collect();

    let analyzers: Vec<Value> = index
        .analyzers()
        .map(|(_, analyzer)| canonicalize_analyzer(analyzer))
        .collect();

    let root = json!({
        "version": SNAPSHOT_VERSION,
        "analyzers": analyzers,
        "tables": tables,
    });
    serde_json::to_string_pretty(&root).expect("canonical snapshot is valid JSON")
}

fn canonicalize_table(index: &SchemaIndex<'_>, table: &Table) -> Value {
    let fields: Vec<Value> = index
        .fields()
        .filter(|(key, _)| key.as_tuple().0 == table.name.as_str())
        .map(|(_, field)| canonicalize_field(field))
        .collect();

    let indexes: Vec<Value> = index
        .indexes()
        .filter(|(key, _)| key.as_tuple().0 == table.name.as_str())
        .map(|(_, index)| canonicalize_index(index))
        .collect();

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
