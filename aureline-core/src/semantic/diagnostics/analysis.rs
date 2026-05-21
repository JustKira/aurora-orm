use super::RenderSemanticDiagnostic;
use crate::semantic::{SemanticDiagnostic, SemanticDiagnosticKind, SemanticError};

#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisDiagnosticKind {
    DuplicateTableName {
        name: String,
    },
    DuplicateNormalizedTableName {
        name: String,
    },
    DuplicateAnalyzerName {
        name: String,
    },
    DuplicateFieldName {
        table: String,
        field: String,
    },
    UnknownRecordTable {
        table: String,
    },
    UnknownAnalyzer {
        name: String,
    },
    ReservedFunctionParam {
        name: String,
    },
    InvalidFunctionBodyParams {
        message: String,
    },
    FunctionBodyParamMismatch {
        function: String,
        missing: Vec<String>,
        unknown: Vec<String>,
    },
    UnknownSurqlVariable {
        variable: String,
    },
}

impl RenderSemanticDiagnostic for AnalysisDiagnosticKind {
    fn message(&self) -> String {
        match self {
            Self::DuplicateTableName { name } => format!("duplicate table name `{name}`"),
            Self::DuplicateNormalizedTableName { name } => {
                format!("duplicate table name `{name}` after normalization")
            }
            Self::DuplicateAnalyzerName { name } => format!("duplicate analyzer name `{name}`"),
            Self::DuplicateFieldName { table, field } => {
                format!("duplicate field name `{field}` on table {table}")
            }
            Self::UnknownRecordTable { table } => format!("unknown record table `{table}`"),
            Self::UnknownAnalyzer { name } => format!("unknown analyzer `{name}`"),
            Self::ReservedFunctionParam { name } => {
                format!("function parameter name `{name}` is reserved")
            }
            Self::InvalidFunctionBodyParams { message } => message.clone(),
            Self::FunctionBodyParamMismatch {
                function,
                missing,
                unknown,
            } => {
                let mut parts = Vec::new();
                if !missing.is_empty() {
                    parts.push(format!(
                        "missing references for function arguments: {}",
                        format_names(missing)
                    ));
                }
                if !unknown.is_empty() {
                    parts.push(format!(
                        "unknown function body parameters: {}",
                        format_names(unknown)
                    ));
                }

                format!(
                    "function `{function}` SurQL body parameters do not match signature: {}",
                    parts.join("; ")
                )
            }
            Self::UnknownSurqlVariable { variable } => {
                format!("unknown SurrealQL variable `${variable}`")
            }
        }
    }

    fn hint(&self) -> Option<String> {
        None
    }
}

fn format_names(names: &[String]) -> String {
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn analysis_diagnostic(kind: AnalysisDiagnosticKind) -> SemanticError {
    SemanticDiagnostic::new(SemanticDiagnosticKind::Analysis(kind))
}

pub(crate) fn duplicate_table_name(name: impl Into<String>) -> SemanticError {
    analysis_diagnostic(AnalysisDiagnosticKind::DuplicateTableName { name: name.into() })
}

pub(crate) fn duplicate_normalized_table_name(name: impl Into<String>) -> SemanticError {
    analysis_diagnostic(AnalysisDiagnosticKind::DuplicateNormalizedTableName { name: name.into() })
}

pub(crate) fn duplicate_analyzer_name(name: impl Into<String>) -> SemanticError {
    analysis_diagnostic(AnalysisDiagnosticKind::DuplicateAnalyzerName { name: name.into() })
}

pub(crate) fn duplicate_field_name(
    table: impl Into<String>,
    field: impl Into<String>,
) -> SemanticError {
    analysis_diagnostic(AnalysisDiagnosticKind::DuplicateFieldName {
        table: table.into(),
        field: field.into(),
    })
}

pub(crate) fn unknown_record_table(table: impl Into<String>) -> SemanticError {
    analysis_diagnostic(AnalysisDiagnosticKind::UnknownRecordTable {
        table: table.into(),
    })
}

pub(crate) fn unknown_analyzer(name: impl Into<String>) -> SemanticError {
    analysis_diagnostic(AnalysisDiagnosticKind::UnknownAnalyzer { name: name.into() })
}

pub(crate) fn reserved_function_param(name: impl Into<String>) -> SemanticError {
    analysis_diagnostic(AnalysisDiagnosticKind::ReservedFunctionParam { name: name.into() })
}

pub(crate) fn invalid_function_body_params(message: impl Into<String>) -> SemanticError {
    analysis_diagnostic(AnalysisDiagnosticKind::InvalidFunctionBodyParams {
        message: message.into(),
    })
}

pub(crate) fn function_body_param_mismatch(
    function: impl Into<String>,
    missing: Vec<String>,
    unknown: Vec<String>,
) -> SemanticError {
    analysis_diagnostic(AnalysisDiagnosticKind::FunctionBodyParamMismatch {
        function: function.into(),
        missing,
        unknown,
    })
}

pub(crate) fn unknown_surql_variable(variable: impl Into<String>) -> SemanticError {
    analysis_diagnostic(AnalysisDiagnosticKind::UnknownSurqlVariable {
        variable: variable.into(),
    })
}
