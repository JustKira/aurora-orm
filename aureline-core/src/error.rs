use crate::check::diagnostics::ParseDiagnostic;
use crate::validate::ValidationError;

#[derive(Debug, thiserror::Error)]
pub enum AurelineError {
    #[error("failed to parse Aureline schema: {0}")]
    Parse(ParseDiagnostic),
    #[error("failed to convert Aureline parse tree: {0}")]
    Convert(String),
    #[error("failed to serialize Aureline AST: {0}")]
    Json(serde_json::Error),
    #[error("validation failed: {}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; "))]
    Validation(Vec<ValidationError>),
}
