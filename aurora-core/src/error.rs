use crate::check::diagnostics::ParseDiagnostic;
use crate::validate::ValidationError;

#[derive(Debug, thiserror::Error)]
pub enum AuroraError {
    #[error("failed to parse Aurora schema: {0}")]
    Parse(ParseDiagnostic),
    #[error("failed to convert Aurora parse tree: {0}")]
    Convert(String),
    #[error("failed to serialize Aurora AST: {0}")]
    Json(serde_json::Error),
    #[error("validation failed: {}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; "))]
    Validation(Vec<ValidationError>),
}
