use aurora_core::ast::{Field, Table};
use aurora_core::emit::{
    emit_alter_field, emit_analyzer, emit_field, emit_index, emit_remove_analyzer,
    emit_remove_field, emit_remove_index, emit_remove_table, emit_table,
};

use crate::ops::Op;

// SurrealDB v3 schema-mutation behavior (verified via tools/surrealdb-probe):
//
// 1. Schema rules are enforced only at write time. Existing rows are *never*
//    touched by ALTER, REMOVE FIELD, DEFINE OVERWRITE, or any field-level DDL.
// 2. As a result, `ALTER FIELD ... TYPE ...` preserves data even when the new
//    type is incompatible with stored values — the row stays as-is and only
//    *future* writes are validated against the new type.
// 3. `REMOVE FIELD` does not clear the underlying field value either; it just
//    removes the schema rule. Data lives in the document.
// 4. `REMOVE TABLE` is the only field/table-level op that physically deletes
//    rows.
//
// Practical implication for this file: for ChangeFieldType / ChangeFieldOptional /
// ChangeFieldFlexible we emit `ALTER FIELD` instead of `REMOVE FIELD` + `DEFINE
// FIELD`. The data outcome is the same (preserved), but the SurrealQL is one
// statement instead of two and reads as intent ("alter") rather than
// implementation detail.
//
// Index and analyzer changes use REMOVE + DEFINE: SurrealDB's `ALTER INDEX`
// only changes COMMENT/CONCURRENTLY/DEFER (none of which Aurora exposes), and
// there is no `ALTER ANALYZER`.
//
// CAVEAT: a type change can leave existing rows in violation of the new schema.
// SurrealDB will silently return them on read but reject any write that touches
// the field with the old shape. Treat type/optional changes as "data
// invalidation" hazards even though they're not "data loss" hazards. See
// `Op::destructive` in ops.rs for related TODO around splitting these.

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
            // Note: indexes attached to a freshly-created table are emitted
            // as separate Op::CreateIndex ops by the diff layer, so we don't
            // need to emit them inline here.
            out
        }
        Op::RemoveTable(table) => vec![emit_remove_table(table)],
        Op::AddField { table, field } => vec![emit_field(table, field)],
        Op::RemoveField { table, field } => vec![emit_remove_field(table, field)],
        Op::ChangeFieldType { table, field, .. }
        | Op::ChangeFieldOptional { table, field, .. }
        | Op::ChangeFieldFlexible { table, field, .. } => {
            vec![emit_alter_field(table, field)]
        }
        Op::ChangeTableMode { table, to, .. } => vec![emit_table(&Table {
            name: table.clone(),
            modifier: to.clone(),
            fields: Vec::new(),
            indexes: Vec::new(),
            raw_attributes: Vec::new(),
        })],
        Op::CreateAnalyzer(a) => vec![emit_analyzer(a)],
        Op::RemoveAnalyzer(name) => vec![emit_remove_analyzer(name)],
        Op::ChangeAnalyzer { from, to } => {
            vec![emit_remove_analyzer(&from.name), emit_analyzer(to)]
        }
        Op::CreateIndex { table, index } => vec![emit_index(table, index)],
        Op::RemoveIndex { table, name } => vec![emit_remove_index(table, name)],
        Op::ChangeIndex {
            table, name, to, ..
        } => vec![emit_remove_index(table, name), emit_index(table, to)],
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
        Op::AddField { table, field } => vec![emit_remove_field(table, &field.name)],
        Op::RemoveField { table, field } => vec![format!(
            "-- down: RemoveField {table}.{field} cannot restore data"
        )],
        Op::ChangeFieldType {
            table,
            field,
            from_type,
        } => {
            let previous = Field {
                name: field.name.clone(),
                ty: from_type.clone(),
                optional: field.optional,
                flexible: field.flexible,
                raw_attributes: Vec::new(),
            };
            vec![emit_alter_field(table, &previous)]
        }
        Op::ChangeFieldOptional {
            table,
            field,
            now_optional,
        } => {
            let previous = Field {
                name: field.name.clone(),
                ty: field.ty.clone(),
                optional: !now_optional,
                flexible: field.flexible,
                raw_attributes: Vec::new(),
            };
            vec![emit_alter_field(table, &previous)]
        }
        Op::ChangeFieldFlexible {
            table,
            field,
            now_flexible,
        } => {
            let previous = Field {
                name: field.name.clone(),
                ty: field.ty.clone(),
                optional: field.optional,
                flexible: !now_flexible,
                raw_attributes: Vec::new(),
            };
            vec![emit_alter_field(table, &previous)]
        }
        Op::ChangeTableMode { table, from, .. } => vec![emit_table(&Table {
            name: table.clone(),
            modifier: from.clone(),
            fields: Vec::new(),
            indexes: Vec::new(),
            raw_attributes: Vec::new(),
        })],
        Op::CreateAnalyzer(a) => vec![emit_remove_analyzer(&a.name)],
        Op::RemoveAnalyzer(name) => vec![format!(
            "-- down: RemoveAnalyzer {name} cannot restore the original definition"
        )],
        Op::ChangeAnalyzer { from, to } => {
            vec![emit_remove_analyzer(&to.name), emit_analyzer(from)]
        }
        Op::CreateIndex { table, index } => vec![emit_remove_index(table, &index.name)],
        Op::RemoveIndex { table, name } => vec![format!(
            "-- down: RemoveIndex {table}.{name} cannot restore the original definition"
        )],
        Op::ChangeIndex {
            table, name, from, ..
        } => vec![emit_remove_index(table, name), emit_index(table, from)],
    }
}

fn join(parts: Vec<String>) -> String {
    if parts.is_empty() {
        String::new()
    } else {
        format!("{}\n", parts.join("\n"))
    }
}
