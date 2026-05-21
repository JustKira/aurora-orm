use super::RenderSemanticDiagnostic;
use super::suggestions::closest_match;
use crate::semantic::{SemanticDiagnostic, SemanticDiagnosticKind, SemanticError};

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
pub enum AttributeDiagnosticKind {
    UnknownAttribute {
        scope: AttributeScope,
        name: String,
        valid: &'static [&'static str],
    },
    InvalidUsage {
        message: String,
    },
}

impl RenderSemanticDiagnostic for AttributeDiagnosticKind {
    fn message(&self) -> String {
        match self {
            Self::UnknownAttribute { scope, name, .. } => {
                format!(
                    "unknown {} attribute `{}{}`",
                    scope.label(),
                    scope.prefix(),
                    name
                )
            }
            Self::InvalidUsage { message } => message.clone(),
        }
    }

    fn hint(&self) -> Option<String> {
        match self {
            Self::UnknownAttribute { scope, name, valid } => closest_match(name, valid)
                .map(|suggestion| format!("did you mean `{}{}`?", scope.prefix(), suggestion)),
            Self::InvalidUsage { .. } => None,
        }
    }
}

pub(crate) fn unknown_attribute(
    scope: AttributeScope,
    name: &str,
    valid: &'static [&'static str],
) -> SemanticError {
    SemanticDiagnostic::new(SemanticDiagnosticKind::Attribute(
        AttributeDiagnosticKind::UnknownAttribute {
            scope,
            name: name.to_string(),
            valid,
        },
    ))
}

pub(crate) fn invalid_attribute_usage(message: impl Into<String>) -> SemanticError {
    SemanticDiagnostic::new(SemanticDiagnosticKind::Attribute(
        AttributeDiagnosticKind::InvalidUsage {
            message: message.into(),
        },
    ))
}
