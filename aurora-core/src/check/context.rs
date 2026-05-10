use pest::error::LineColLocation;

use super::keywords::{ANALYZER, TABLE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SyntaxContext {
    MissingBlockStart(BlockKind),
    GeometryType,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlockKind {
    Analyzer,
    Table,
}

impl BlockKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Analyzer => ANALYZER,
            Self::Table => TABLE,
        }
    }
}

impl SyntaxContext {
    // Pest can only tell us the missing rule, e.g. `identifier`. This tiny
    // source scan lets diagnostics phrase that rule by local context without
    // adding diagnostic-only rules to the grammar or converter.
    pub(super) fn from_error(error: &pest::error::Error<crate::grammar::Rule>) -> Self {
        let line = error.line();
        let column = match error.line_col {
            LineColLocation::Pos((_, column)) => column,
            LineColLocation::Span((_, column), _) => column,
        };
        let before_error = before_column(line, column);

        if let Some(block_kind) = missing_block_start(line) {
            return Self::MissingBlockStart(block_kind);
        }

        if before_error.contains("geometry<") {
            return Self::GeometryType;
        }

        Self::Unknown
    }

    pub(super) fn missing_block_start(self) -> Option<BlockKind> {
        match self {
            Self::MissingBlockStart(block_kind) => Some(block_kind),
            _ => None,
        }
    }
}

fn before_column(line: &str, column: usize) -> &str {
    let end = column.saturating_sub(1).min(line.len());
    &line[..end]
}

fn missing_block_start(before_error: &str) -> Option<BlockKind> {
    let trimmed = before_error.trim_start();
    if trimmed.contains('{') {
        return None;
    }

    if starts_with_declaration_keyword(trimmed, TABLE) {
        return Some(BlockKind::Table);
    }

    if starts_with_declaration_keyword(trimmed, ANALYZER) {
        return Some(BlockKind::Analyzer);
    }

    None
}

fn starts_with_declaration_keyword(text: &str, keyword: &str) -> bool {
    text.strip_prefix(keyword)
        .and_then(|rest| rest.chars().next())
        .is_some_and(char::is_whitespace)
}
