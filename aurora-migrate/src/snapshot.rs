use aurora_core::ast::{Analyzer, Field, Index, Schema, SchemaItem, Table, Type};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::{Error, Result};

pub const SNAPSHOT_VERSION: u32 = 1;

pub fn canonicalize(schema: &Schema) -> String {
    let tables: Vec<Value> = sorted_tables(schema)
        .iter()
        .map(|table| canonicalize_table(table))
        .collect();

    let analyzers: Vec<Value> = sorted_analyzers(schema)
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

/// Returns the schema's tables in alphabetical order, dropping doc comments.
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

pub fn parse_snapshot(json_str: &str) -> Result<Schema> {
    let snapshot = serde_json::from_str::<Snapshot>(json_str)
        .map_err(|error| Error::SnapshotDecode(error.to_string()))?;

    match snapshot.version {
        SNAPSHOT_VERSION => snapshot_v1_to_schema(snapshot),
        version => Err(Error::SnapshotDecode(format!(
            "unsupported snapshot version {version}"
        ))),
    }
}

fn snapshot_v1_to_schema(snapshot: Snapshot) -> Result<Schema> {
    let mut items = Vec::new();
    for a in snapshot.analyzers {
        items.push(SchemaItem::AnalyzerDecl(a));
    }
    for t in snapshot.tables {
        items.push(SchemaItem::TableDecl(t.into_ast()));
    }
    Ok(Schema { items })
}

#[derive(Deserialize)]
struct Snapshot {
    version: u32,
    #[serde(default)]
    analyzers: Vec<Analyzer>,
    tables: Vec<SnapshotTable>,
}

#[derive(Deserialize)]
struct SnapshotTable {
    name: String,
    modifier: Option<String>,
    fields: Vec<SnapshotField>,
    #[serde(default)]
    indexes: Vec<Index>,
}

impl SnapshotTable {
    fn into_ast(self) -> Table {
        Table {
            name: self.name,
            modifier: self.modifier,
            fields: self
                .fields
                .into_iter()
                .map(|field| Field {
                    name: field.name,
                    ty: field.ty,
                    optional: field.optional,
                    flexible: field.flexible,
                    raw_attributes: Vec::new(),
                })
                .collect(),
            indexes: self.indexes,
            raw_attributes: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
struct SnapshotField {
    name: String,
    #[serde(rename = "type")]
    ty: Type,
    optional: bool,
    #[serde(default)]
    flexible: bool,
}
