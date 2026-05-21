use std::fmt;

use crate::check::diagnostics::SourceRange;

use super::diagnostics::closest_match;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeScope {
    Field,
    Block,
    FunctionBlock,
}

impl AttributeScope {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Field => "field",
            Self::Block => "block",
            Self::FunctionBlock => "function block",
        }
    }

    pub(crate) fn prefix(self) -> &'static str {
        match self {
            Self::Field => "@",
            Self::Block | Self::FunctionBlock => "@@",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticDiagnosticKind {
    UnknownAttribute {
        scope: AttributeScope,
        name: String,
        valid: &'static [&'static str],
    },
}

impl SemanticDiagnosticKind {
    pub fn message(&self) -> String {
        match self {
            Self::UnknownAttribute { scope, name, .. } => {
                format!(
                    "unknown {} attribute `{}{}`",
                    scope.label(),
                    scope.prefix(),
                    name
                )
            }
        }
    }

    pub fn hint(&self) -> Option<String> {
        match self {
            Self::UnknownAttribute { scope, name, valid } => closest_match(name, valid)
                .map(|suggestion| format!("did you mean `{}{}`?", scope.prefix(), suggestion)),
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
