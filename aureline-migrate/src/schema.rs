use aureline_core::ast::{Field, Schema, SchemaItem, Table};

/// The migration engine is being rebuilt incrementally. This first slice only
/// supports tables and fields, so snapshots and diffs must not record indexes,
/// analyzers, raw attributes, or raw SurrealQL as migrated state.
pub fn table_field_schema(schema: &Schema) -> Schema {
    Schema {
        items: schema
            .items
            .iter()
            .filter_map(|item| match item {
                SchemaItem::TableDecl(table) => {
                    Some(SchemaItem::TableDecl(table_field_table(table)))
                }
                SchemaItem::DocComment { .. }
                | SchemaItem::SurqlBlock(_)
                | SchemaItem::AnalyzerDecl(_) => None,
            })
            .collect(),
    }
}

pub(crate) fn table_field_table(table: &Table) -> Table {
    Table {
        name: table.name.clone(),
        modifier: table.modifier.clone(),
        fields: table.fields.iter().map(table_field_field).collect(),
        indexes: Vec::new(),
        raw_attributes: Vec::new(),
    }
}

pub(crate) fn table_field_field(field: &Field) -> Field {
    Field {
        name: field.name.clone(),
        ty: field.ty.clone(),
        optional: field.optional,
        flexible: field.flexible,
        raw_attributes: Vec::new(),
    }
}
