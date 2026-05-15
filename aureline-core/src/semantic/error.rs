use std::fmt;

use crate::check::diagnostics::SourceRange;

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticError {
    pub message: String,
    /// Human-readable hint, e.g. "did you mean `@hnsw`?"
    pub hint: Option<String>,
    pub range: Option<SourceRange>,
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(hint) = &self.hint {
            write!(f, " - {hint}")?;
        }
        Ok(())
    }
}

pub type SemanticResult<T = ()> = Result<T, Vec<SemanticError>>;
