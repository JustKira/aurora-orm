use aureline_core::ast::{Field, Schema, SchemaItem, Table};
use aureline_core::schema_index::SchemaIndex;

/// The migration engine is being rebuilt incrementally. This first slice only
/// supports tables and fields, so snapshots and diffs must not record indexes,
/// analyzers, raw attributes, or raw SurrealQL as migrated state.
pub fn table_field_schema(schema: &Schema) -> Schema {
    let index = SchemaIndex::from_schema(schema);

    Schema {
        items: index
            .tables()
            .map(|(_, table)| SchemaItem::TableDecl(table_field_table(table)))
            .collect(),
    }
}

pub(crate) fn table_field_table(table: &Table) -> Table {
    Table {
        name: table.name.clone(),
        source_range: table.source_range,
        name_range: table.name_range,
        modifier: table.modifier.clone(),
        fields: table.fields.iter().map(table_field_field).collect(),
        indexes: Vec::new(),
        raw_attributes: Vec::new(),
    }
}

pub(crate) fn table_field_field(field: &Field) -> Field {
    Field {
        name: field.name.clone(),
        source_range: field.source_range,
        name_range: field.name_range,
        ty: field.ty.clone(),
        optional: field.optional,
        flexible: field.flexible,
        always: field.always,
        default: field.default.clone(),
        raw_attributes: Vec::new(),
    }
}

/// Full schema preserves analyzers and indexes while stripping DocComment and raw_attributes.
/// Used for diffing and canonicalization where full schema fidelity is needed.
pub fn full_schema(schema: &Schema) -> Schema {
    let index = SchemaIndex::from_schema(schema);

    Schema {
        items: index
            .analyzers()
            .map(|(_, analyzer)| SchemaItem::AnalyzerDecl(analyzer.clone()))
            .chain(
                index
                    .tables()
                    .map(|(_, table)| SchemaItem::TableDecl(full_table(table))),
            )
            .collect(),
    }
}

fn full_table(table: &Table) -> Table {
    Table {
        name: table.name.clone(),
        source_range: table.source_range,
        name_range: table.name_range,
        modifier: table.modifier.clone(),
        fields: table.fields.iter().map(full_field).collect(),
        indexes: table.indexes.clone(),
        // Strip raw_attributes as they're intermediate representation
        raw_attributes: Vec::new(),
    }
}

fn full_field(field: &Field) -> Field {
    Field {
        name: field.name.clone(),
        source_range: field.source_range,
        name_range: field.name_range,
        ty: field.ty.clone(),
        optional: field.optional,
        flexible: field.flexible,
        always: field.always,
        default: field.default.clone(),
        // Strip raw_attributes as they're intermediate representation
        raw_attributes: Vec::new(),
    }
}
