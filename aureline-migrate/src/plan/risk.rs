use crate::change::FieldChangeSet;
use crate::ops::Op;
use aureline_core::emit::surql_type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRisk {
    pub step_index: usize,
    pub kind: RiskKind,
    pub message: String,
}

impl PlanRisk {
    pub fn blocks_by_default(&self) -> bool {
        matches!(
            self.kind,
            RiskKind::DataLoss | RiskKind::DataInvalidation | RiskKind::IrreversibleDown
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskKind {
    DataLoss,
    DataInvalidation,
    IrreversibleDown,
}

pub fn analyze_risks(steps: &[Op]) -> Vec<PlanRisk> {
    steps
        .iter()
        .enumerate()
        .flat_map(|(step_index, step)| risks_for_step(step_index, step))
        .collect()
}

fn risks_for_step(step_index: usize, step: &Op) -> Vec<PlanRisk> {
    match step {
        Op::RemoveTable(table) => vec![PlanRisk {
            step_index,
            kind: RiskKind::DataLoss,
            message: format!("RemoveTable {table}: removes all records in the table"),
        }],
        Op::RemoveField { table, field } => vec![PlanRisk {
            step_index,
            kind: RiskKind::IrreversibleDown,
            message: format!(
                "RemoveField {table}.{}: down cannot restore the removed field definition",
                field.name
            ),
        }],
        Op::AlterField {
            table,
            from,
            to,
            changes: FieldChangeSet {
                type_changed: true, ..
            },
        } => vec![PlanRisk {
            step_index,
            kind: RiskKind::DataInvalidation,
            message: format!(
                "AlterField {table}.{}: changing type from {} to {} may invalidate existing records",
                to.name,
                surql_type(&from.ty),
                surql_type(&to.ty)
            ),
        }],
        Op::AlterField {
            table,
            to,
            changes:
                FieldChangeSet {
                    optional_changed: true,
                    ..
                },
            ..
        } if !to.optional => vec![PlanRisk {
            step_index,
            kind: RiskKind::DataInvalidation,
            message: format!(
                "AlterField {table}.{}: changing optional to required may invalidate existing records",
                to.name
            ),
        }],
        _ => Vec::new(),
    }
}
