use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: DiagnosticCode,
    pub message: String,
    pub range: SourceRange,
    pub help: Vec<String>,
    pub data: Option<DiagnosticData>,
}

impl Diagnostic {
    pub fn error(code: DiagnosticCode, message: impl Into<String>, range: SourceRange) -> Self {
        Self {
            severity: Severity::Error,
            code,
            message: message.into(),
            range,
            help: Vec::new(),
            data: None,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help.push(help.into());
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        for help in &self.help {
            write!(f, "\nhelp: {help}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

impl Severity {
    pub fn is_error(self) -> bool {
        matches!(self, Severity::Error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    ParseError,
    ValidationError,
    ConvertError,
}

impl DiagnosticCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ParseError => "parse_error",
            Self::ValidationError => "validation_error",
            Self::ConvertError => "convert_error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum DiagnosticData {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRange {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

impl SourceRange {
    pub fn first_character() -> Self {
        // Fallback diagnostics do not know the source text, so prefer a safe
        // zero-width origin range. Using `0..1` can be out-of-bounds for empty
        // documents, and some LSP clients reject ranges past the line length.
        Self {
            start: SourcePosition {
                line: 0,
                character: 0,
            },
            end: SourcePosition {
                line: 0,
                character: 0,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    pub line: u32,
    pub character: u32,
}

pub type ParseDiagnostic = Diagnostic;
