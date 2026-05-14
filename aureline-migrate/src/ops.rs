use aureline_core::ast::{Field, Table};
use aureline_core::emit::surql_type;

use crate::change::FieldChangeSet;

/// Executable migration steps after planning.
///
/// The first planner slice supports tables and fields only. Indexes, analyzers,
/// and other schema entities will be added back as explicit planner steps.
///
/// TODO(migrate): table/field support currently tracks table mode plus field
/// type/optional/flexible only. SurrealDB also exposes table TYPE, DROP,
/// permissions, changefeeds, views, comments, and field defaults, values,
/// readonly, assertions, permissions, references, computed fields, and comments.
/// Add each clause as a structured change before preserving it in migrations.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// Creates a table definition and emits its current field definitions.
    CreateTable(Table),
    /// Removes a table definition; SurrealDB also deletes the table records.
    RemoveTable(String),
    /// Changes table mode, for example schemaless <-> schemafull or drop mode.
    ChangeTableMode {
        table: String,
        from: Option<String>,
        to: Option<String>,
    },
    /// Adds a schema field definition to an existing table.
    AddField { table: String, field: Field },
    /// Removes a field definition; SurrealDB keeps existing stored values.
    RemoveField { table: String, field: Field },
    /// Alters tracked field clauses only; existing rows are not rewritten.
    AlterField {
        table: String,
        from: Field,
        to: Field,
        changes: FieldChangeSet,
    },
}

impl Op {
    pub fn destructive(&self) -> bool {
        matches!(
            self,
            Op::RemoveTable(_)
                | Op::RemoveField { .. }
                | Op::AlterField {
                    changes: FieldChangeSet {
                        type_changed: true,
                        ..
                    },
                    ..
                }
                | Op::AlterField {
                    to: Field {
                        optional: false,
                        ..
                    },
                    changes: FieldChangeSet {
                        optional_changed: true,
                        ..
                    },
                    ..
                }
        )
    }

    pub fn summary(&self) -> String {
        match self {
            Op::CreateTable(table) => format!("+ CREATE TABLE {}", table.name),
            Op::RemoveTable(table) => format!("- REMOVE TABLE {table}"),
            Op::ChangeTableMode { table, from, to } => {
                format!("~ CHANGE TABLE MODE {table} {:?} -> {:?}", from, to)
            }
            Op::AddField { table, field } => format!("+ ADD FIELD {table}.{}", field.name),
            Op::RemoveField { table, field } => {
                format!("- REMOVE FIELD {table}.{}", field.name)
            }
            Op::AlterField {
                table,
                from,
                to,
                changes,
            } => alter_field_summary(table, from, to, *changes),
        }
    }
}

fn alter_field_summary(table: &str, from: &Field, to: &Field, changes: FieldChangeSet) -> String {
    if changes.type_changed {
        return format!(
            "~ CHANGE TYPE {table}.{} {} -> {}",
            to.name,
            surql_type(&from.ty),
            surql_type(&to.ty)
        );
    }
    if changes.optional_changed {
        return format!(
            "~ CHANGE OPTIONAL {table}.{} -> {}",
            to.name,
            if to.optional { "optional" } else { "required" }
        );
    }
    if changes.flexible_changed {
        return format!(
            "~ CHANGE FLEXIBLE {table}.{} -> {}",
            to.name,
            if to.flexible { "flexible" } else { "strict" }
        );
    }
    format!("~ ALTER FIELD {table}.{}", to.name)
}
