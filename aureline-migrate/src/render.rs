use aureline_core::ast::Table;
use aureline_core::emit::{
    emit_alter_field, emit_field, emit_remove_field, emit_remove_table, emit_table,
};

use crate::ops::Op;

// SurrealDB v3 schema-mutation behavior verified for the current table/field
// slice:
//
// 1. ALTER FIELD preserves stored values but can make future writes fail.
// 2. REMOVE FIELD removes the schema rule, not the stored values.
// 3. REMOVE TABLE is the table/field operation that physically deletes rows.
//
// The planner owns which steps are emitted and their risk classification. This
// renderer only turns planned steps into SurrealQL.

pub fn emit_up(ops: &[Op]) -> String {
    let parts = ops.iter().flat_map(emit_up_op).collect::<Vec<_>>();
    join(parts)
}

pub fn emit_down(ops: &[Op]) -> String {
    let parts = ops.iter().rev().flat_map(emit_down_op).collect::<Vec<_>>();
    join(parts)
}

fn emit_up_op(op: &Op) -> Vec<String> {
    match op {
        Op::CreateTable(table) => {
            let mut out = vec![emit_table(table)];
            let mut fields = table.fields.iter().collect::<Vec<_>>();
            fields.sort_by(|a, b| a.name.cmp(&b.name));
            out.extend(
                fields
                    .into_iter()
                    .map(|field| emit_field(&table.name, field)),
            );
            out
        }
        Op::RemoveTable(table) => vec![emit_remove_table(table)],
        Op::ChangeTableMode { table, to, .. } => vec![emit_table(&Table {
            name: table.clone(),
            modifier: to.clone(),
            fields: Vec::new(),
            indexes: Vec::new(),
            raw_attributes: Vec::new(),
        })],
        Op::AddField { table, field } => vec![emit_field(table, field)],
        Op::RemoveField { table, field } => vec![emit_remove_field(table, &field.name)],
        Op::AlterField { table, to, .. } => vec![emit_alter_field(table, to)],
    }
}

fn emit_down_op(op: &Op) -> Vec<String> {
    match op {
        Op::CreateTable(table) => vec![emit_remove_table(&table.name)],
        Op::RemoveTable(table) => vec![format!(
            "-- down: RemoveTable {table} cannot restore data\n{}",
            emit_table(&Table {
                name: table.clone(),
                modifier: None,
                fields: Vec::new(),
                indexes: Vec::new(),
                raw_attributes: Vec::new(),
            })
        )],
        Op::ChangeTableMode { table, from, .. } => vec![emit_table(&Table {
            name: table.clone(),
            modifier: from.clone(),
            fields: Vec::new(),
            indexes: Vec::new(),
            raw_attributes: Vec::new(),
        })],
        Op::AddField { table, field } => vec![emit_remove_field(table, &field.name)],
        Op::RemoveField { table, field } => vec![format!(
            "-- down: RemoveField {table}.{} cannot restore data",
            field.name
        )],
        Op::AlterField { table, from, .. } => vec![emit_alter_field(table, from)],
    }
}

fn join(parts: Vec<String>) -> String {
    if parts.is_empty() {
        String::new()
    } else {
        format!("{}\n", parts.join("\n"))
    }
}
