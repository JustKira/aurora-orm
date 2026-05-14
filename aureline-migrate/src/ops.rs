use aureline_core::ast::{Analyzer, Field, Index, Table, Type};
use aureline_core::emit::surql_type;

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    CreateTable(Table),
    RemoveTable(String),
    AddField {
        table: String,
        field: Field,
    },
    RemoveField {
        table: String,
        field: String,
    },
    ChangeFieldType {
        table: String,
        field: Field,
        from_type: Type,
    },
    ChangeFieldOptional {
        table: String,
        field: Field,
        now_optional: bool,
    },
    ChangeFieldFlexible {
        table: String,
        field: Field,
        now_flexible: bool,
    },
    ChangeTableMode {
        table: String,
        from: Option<String>,
        to: Option<String>,
    },
    CreateAnalyzer(Analyzer),
    RemoveAnalyzer(String),
    /// Analyzer changes: SurrealQL has no `ALTER ANALYZER`, so renderer emits
    /// REMOVE + DEFINE in a single migration step.
    ChangeAnalyzer {
        from: Analyzer,
        to: Analyzer,
    },
    CreateIndex {
        table: String,
        index: Index,
    },
    RemoveIndex {
        table: String,
        name: String,
    },
    /// Index changes: SurrealQL `ALTER INDEX` only changes COMMENT/CONCURRENTLY/
    /// DEFER (none of which Aureline exposes), so any real change becomes
    /// REMOVE + DEFINE in the renderer.
    ChangeIndex {
        table: String,
        name: String,
        from: Index,
        to: Index,
    },
}

impl Op {
    // TODO: This conflates two different hazards. Per surrealdb-probe findings,
    // SurrealDB v3 enforces schema only at write time:
    //   * Only `RemoveTable` physically deletes data.
    //   * `RemoveField`, `ChangeFieldType`, `ChangeFieldOptional → required` all
    //     leave existing data alive but possibly invalid under the new schema.
    //   * Index/analyzer ops never touch data but can break running queries.
    // Consider splitting into `data_loss()` (just RemoveTable) and
    // `data_invalidation()` (the others) so the apply tool can warn distinctly.
    pub fn destructive(&self) -> bool {
        matches!(
            self,
            Op::RemoveTable(_)
                | Op::RemoveField { .. }
                | Op::ChangeFieldType { .. }
                | Op::ChangeFieldOptional {
                    now_optional: false,
                    ..
                }
        )
    }

    pub fn summary(&self) -> String {
        match self {
            Op::CreateTable(table) => format!("+ CREATE TABLE {}", table.name),
            Op::RemoveTable(table) => format!("- REMOVE TABLE {table}"),
            Op::AddField { table, field } => format!("+ ADD FIELD {table}.{}", field.name),
            Op::RemoveField { table, field } => format!("- REMOVE FIELD {table}.{field}"),
            Op::ChangeFieldType {
                table,
                field,
                from_type,
            } => format!(
                "~ CHANGE TYPE {table}.{} {} -> {}",
                field.name,
                surql_type(from_type),
                surql_type(&field.ty)
            ),
            Op::ChangeFieldOptional {
                table,
                field,
                now_optional,
            } => format!(
                "~ CHANGE OPTIONAL {table}.{} -> {}",
                field.name,
                if *now_optional {
                    "optional"
                } else {
                    "required"
                }
            ),
            Op::ChangeFieldFlexible {
                table,
                field,
                now_flexible,
            } => format!(
                "~ CHANGE FLEXIBLE {table}.{} -> {}",
                field.name,
                if *now_flexible { "flexible" } else { "strict" }
            ),
            Op::ChangeTableMode { table, from, to } => {
                format!("~ CHANGE TABLE MODE {table} {:?} -> {:?}", from, to)
            }
            Op::CreateAnalyzer(a) => format!("+ CREATE ANALYZER {}", a.name),
            Op::RemoveAnalyzer(name) => format!("- REMOVE ANALYZER {name}"),
            Op::ChangeAnalyzer { from, to: _ } => {
                format!("~ CHANGE ANALYZER {}", from.name)
            }
            Op::CreateIndex { table, index } => {
                format!("+ CREATE INDEX {}.{}", table, index.name)
            }
            Op::RemoveIndex { table, name } => format!("- REMOVE INDEX {table}.{name}"),
            Op::ChangeIndex { table, name, .. } => format!("~ CHANGE INDEX {table}.{name}"),
        }
    }
}
