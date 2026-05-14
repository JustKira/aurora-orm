mod risk;

pub use risk::{PlanRisk, RiskKind, analyze_risks};

use crate::change::Change;
use crate::ops::Op;
use crate::schema::{table_field_field, table_field_table};

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationPlan {
    pub steps: Vec<Op>,
    pub risks: Vec<PlanRisk>,
}

pub fn plan_changes(changes: Vec<Change>) -> MigrationPlan {
    let steps = changes.into_iter().map(plan_change).collect::<Vec<_>>();
    let risks = analyze_risks(&steps);
    MigrationPlan { steps, risks }
}

fn plan_change(change: Change) -> Op {
    match change {
        Change::TableAdded(table) => Op::CreateTable(table_field_table(&table)),
        Change::TableRemoved(table) => Op::RemoveTable(table),
        Change::TableModeChanged { table, from, to } => Op::ChangeTableMode { table, from, to },
        Change::FieldAdded { table, field } => Op::AddField {
            table,
            field: table_field_field(&field),
        },
        Change::FieldRemoved { table, field } => Op::RemoveField {
            table,
            field: table_field_field(&field),
        },
        Change::FieldChanged {
            table,
            from,
            to,
            changes,
        } => Op::AlterField {
            table,
            from: table_field_field(&from),
            to: table_field_field(&to),
            changes,
        },
    }
}
