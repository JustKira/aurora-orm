use pest::error::LineColLocation;

use super::diagnostics::{SourcePosition, SourceRange};

// Convert pest's location shape into the SourceRange we expose to the CLI/LSP.
// Pest sometimes gives a full span, but most syntax failures are a single
// point, so this also decides how much source text should be highlighted.
pub(super) fn pest_error_range(
    error: &pest::error::Error<crate::grammar::Rule>,
    highlight_line_end: bool,
) -> SourceRange {
    match error.line_col {
        LineColLocation::Pos(pos) => {
            if highlight_line_end {
                end_of_line_range(error.line(), pos.0)
            } else {
                token_or_single_character_range(error.line(), pos)
            }
        }
        LineColLocation::Span(start, end) => SourceRange {
            start: to_position(start),
            end: to_position(end),
        },
    }
}

// For point errors, prefer highlighting the whole token under the cursor. If
// the cursor is on punctuation/whitespace, fall back to a one-character range.
fn token_or_single_character_range(line: &str, pos: (usize, usize)) -> SourceRange {
    expand_token_range(line, pos).unwrap_or_else(|| single_character_range(pos))
}

// Expand a 1-based pest position to the full ASCII identifier-like token around
// it. This is what turns `duratio` from a one-character underline into a full
// word highlight.
// In simpler terms just walks to the Left and Right to capture the Token.
fn expand_token_range(line: &str, pos: (usize, usize)) -> Option<SourceRange> {
    let column = pos.1.saturating_sub(1);
    let bytes = line.as_bytes();

    // If the error column is outside the line, or if the character under the cursor is not word-like, give up.
    // So if pest points at `{`, `<`, `>`, space, etc., this function returns `None`.
    if column >= bytes.len() || !is_token_byte(bytes[column]) {
        return None;
    }

    let mut start = column;
    while start > 0 && is_token_byte(bytes[start - 1]) {
        start -= 1;
    }

    let mut end = column;
    while end < bytes.len() && is_token_byte(bytes[end]) {
        end += 1;
    }

    Some(SourceRange {
        start: SourcePosition {
            line: pos.0.saturating_sub(1) as u32,
            character: start as u32,
        },
        end: SourcePosition {
            line: pos.0.saturating_sub(1) as u32,
            character: end as u32,
        },
    })
}

// Token expansion is intentionally conservative for now: identifiers and
// keyword-like values are ASCII words plus `_`; punctuation is not included.
fn is_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

// Editors render empty diagnostics inconsistently, so a point error always
// becomes at least a one-character range.
fn single_character_range(pos: (usize, usize)) -> SourceRange {
    let start = to_position(pos);
    SourceRange {
        start,
        end: SourcePosition {
            line: start.line,
            character: start.character.saturating_add(1),
        },
    }
}

// Missing block openers are best shown at the insertion point: the end of the
// current line, not on the previous valid token.
fn end_of_line_range(line: &str, line_number: usize) -> SourceRange {
    let character = line.chars().count() as u32;
    SourceRange {
        start: SourcePosition {
            line: line_number.saturating_sub(1) as u32,
            character,
        },
        end: SourcePosition {
            line: line_number.saturating_sub(1) as u32,
            character: character.saturating_add(1),
        },
    }
}

// Pest uses 1-based line/column positions; SourceRange follows LSP-style
// 0-based line/character positions.
fn to_position((line, column): (usize, usize)) -> SourcePosition {
    SourcePosition {
        line: line.saturating_sub(1) as u32,
        character: column.saturating_sub(1) as u32,
    }
}
