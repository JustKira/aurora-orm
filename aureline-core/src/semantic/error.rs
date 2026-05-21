use std::fmt;

use crate::check::diagnostics::SourceRange;

use super::diagnostics::{
    AnalysisDiagnosticKind, AttributeDiagnosticKind, RenderSemanticDiagnostic,
};

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticDiagnosticKind {
    Analysis(AnalysisDiagnosticKind),
    Attribute(AttributeDiagnosticKind),
}

impl SemanticDiagnosticKind {
    pub fn message(&self) -> String {
        match self {
            Self::Analysis(diagnostic) => diagnostic.message(),
            Self::Attribute(diagnostic) => diagnostic.message(),
        }
    }

    pub fn hint(&self) -> Option<String> {
        match self {
            Self::Analysis(diagnostic) => diagnostic.hint(),
            Self::Attribute(diagnostic) => diagnostic.hint(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticDiagnostic {
    pub kind: SemanticDiagnosticKind,
    pub range: Option<SourceRange>,
}

impl SemanticDiagnostic {
    pub fn new(kind: SemanticDiagnosticKind) -> Self {
        Self { kind, range: None }
    }

    pub fn at(mut self, range: Option<SourceRange>) -> Self {
        self.range = range;
        self
    }

    pub fn range(&self) -> Option<SourceRange> {
        self.range
    }

    pub fn message(&self) -> String {
        self.kind.message()
    }

    pub fn hint(&self) -> Option<String> {
        self.kind.hint()
    }
}

impl fmt::Display for SemanticDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())?;
        if let Some(hint) = self.hint() {
            write!(f, " - {hint}")?;
        }
        Ok(())
    }
}

pub type SemanticError = SemanticDiagnostic;
pub type SemanticResult<T = ()> = Result<T, Vec<SemanticDiagnostic>>;
