use aureline_core::ast::{Analyzer, Field, Index, Schema, SchemaItem, Table, Type};
use serde::Deserialize;

use crate::error::Result;

#[derive(Deserialize)]
pub(super) struct Snapshot {
    pub version: u32,
    #[serde(default)]
    pub analyzers: Vec<Analyzer>,
    pub tables: Vec<SnapshotTable>,
}

#[derive(Deserialize)]
pub(super) struct SnapshotTable {
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
            source_range: None,
            name_range: None,
            modifier: self.modifier,
            fields: self
                .fields
                .into_iter()
                .map(|field| Field {
                    name: field.name,
                    source_range: None,
                    name_range: None,
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

pub(super) fn snapshot_v1_to_schema(snapshot: Snapshot) -> Result<Schema> {
    let mut items = Vec::new();
    for a in snapshot.analyzers {
        items.push(SchemaItem::AnalyzerDecl(a));
    }
    for t in snapshot.tables {
        items.push(SchemaItem::TableDecl(t.into_ast()));
    }
    Ok(Schema { items })
}
