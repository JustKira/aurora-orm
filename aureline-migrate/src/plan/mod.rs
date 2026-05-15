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
    let steps = changes
        .into_iter()
        .flat_map(plan_change)
        .collect::<Vec<_>>();
    let risks = analyze_risks(&steps);
    MigrationPlan { steps, risks }
}

fn plan_change(change: Change) -> Vec<Op> {
    match change {
        Change::TableAdded(table) => vec![Op::CreateTable(table_field_table(&table))],
        Change::TableRemoved(table) => vec![Op::RemoveTable(table)],
        Change::TableModeChanged { table, from, to } => {
            vec![Op::ChangeTableMode { table, from, to }]
        }
        Change::FieldAdded { table, field } => vec![Op::AddField {
            table,
            field: table_field_field(&field),
        }],
        Change::FieldRemoved { table, field } => vec![Op::RemoveField {
            table,
            field: table_field_field(&field),
        }],
        Change::FieldChanged {
            table,
            from,
            to,
            changes,
        } => vec![Op::AlterField {
            table,
            from: table_field_field(&from),
            to: table_field_field(&to),
            changes,
        }],
        Change::AnalyzerAdded(analyzer) => vec![Op::DefineAnalyzer(analyzer)],
        Change::AnalyzerRemoved(analyzer) => vec![Op::RemoveAnalyzer(analyzer)],
        Change::AnalyzerChanged { from, to } => {
            vec![Op::RemoveAnalyzer(from), Op::DefineAnalyzer(to)]
        }
        Change::IndexAdded { table, index } => vec![Op::DefineIndex { table, index }],
        Change::IndexRemoved { table, index } => vec![Op::RemoveIndex {
            table,
            index: index.clone(),
        }],
        Change::IndexChanged { table, from, to } => vec![
            Op::RemoveIndex {
                table: table.clone(),
                index: from,
            },
            Op::DefineIndex { table, index: to },
        ],
    }
}
