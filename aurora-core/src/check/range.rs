use pest::error::LineColLocation;

use super::diagnostics::{SourcePosition, SourceRange};

// Pest reports hard parser errors as either:
// - a point: "the parser got stuck at line/column X"
// - a span: "this exact source span is invalid"
//
// Our diagnostics expose LSP-style line/character ranges, but it is easier to
// reason about point errors in two steps:
// 1. turn the Pest point into a byte span on the current line
// 2. convert that byte span into a SourceRange
#[derive(Debug, Clone, Copy)]
struct LineColumn {
    // Pest line/column values are 1-based.
    line: usize,
    column: usize,
}

#[derive(Debug, Clone, Copy)]
struct LineByteSpan {
    // `line` is still Pest's 1-based line number.
    line: usize,
    // `start` and `end` are byte offsets within that one line.
    start: usize,
    end: usize,
}

// Main entry point for hard Pest parse errors.
//
// `highlight_line_end` is used for missing block openers. In that case the
// useful diagnostic location is the insertion point at end-of-line, not the
// previous token Pest happened to stop near.
pub(super) fn pest_error_range(
    error: &pest::error::Error<crate::grammar::Rule>,
    highlight_line_end: bool,
) -> SourceRange {
    let span = match error.line_col {
        LineColLocation::Pos(pos) => point_error_span(error.line(), pos.into(), highlight_line_end),
        LineColLocation::Span(start, end) => {
            return SourceRange {
                start: pest_position_to_source_position(start.into()),
                end: pest_position_to_source_position(end.into()),
            };
        }
    };

    line_byte_span_to_source_range(span)
}

// Point errors are zero-width from Pest's perspective. We choose a visible
// highlight range for editor/CLI ergonomics:
// - missing `{`: end of line insertion point
// - identifier-like token: the full token
// - punctuation/whitespace: one character
fn point_error_span(
    line_text: &str,
    position: LineColumn,
    highlight_line_end: bool,
) -> LineByteSpan {
    if highlight_line_end {
        return end_of_line_span(line_text, position.line);
    }

    token_span_at_position(line_text, position).unwrap_or_else(|| one_character_span(position))
}

// If Pest points inside `duratio`, highlight the whole `duratio` token instead
// of just one character. This is intentionally ASCII-only today because Aurora
// identifiers are ASCII in the grammar.
fn token_span_at_position(line_text: &str, position: LineColumn) -> Option<LineByteSpan> {
    let column = pest_column_to_byte_index(position.column);
    let bytes = line_text.as_bytes();

    // If Pest points at whitespace/punctuation, there is no word token to
    // expand. The caller will fall back to a one-character span.
    if column >= bytes.len() || !is_token_byte(bytes[column]) {
        return None;
    }

    Some(LineByteSpan {
        line: position.line,
        start: token_start(bytes, column),
        end: token_end(bytes, column),
    })
}

// Walk left from the Pest error byte until the token boundary.
fn token_start(bytes: &[u8], column: usize) -> usize {
    let mut start = column;
    while start > 0 && is_token_byte(bytes[start - 1]) {
        start -= 1;
    }
    start
}

// Walk right from the Pest error byte until the token boundary.
fn token_end(bytes: &[u8], column: usize) -> usize {
    let mut end = column;
    while end < bytes.len() && is_token_byte(bytes[end]) {
        end += 1;
    }
    end
}

// "Token" here means the simple word-like pieces Aurora currently parses:
// identifiers and keywords made from ASCII letters/digits/underscore.
fn is_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

// LSP clients can render zero-width diagnostics inconsistently. If we cannot
// identify a word token, highlight one character at the Pest point.
fn one_character_span(position: LineColumn) -> LineByteSpan {
    let start = pest_column_to_byte_index(position.column);
    LineByteSpan {
        line: position.line,
        start,
        end: start.saturating_add(1),
    }
}

// Missing `{` is clearer as "insert here" at the end of the header line.
fn end_of_line_span(line_text: &str, line: usize) -> LineByteSpan {
    let end = line_text.len();
    LineByteSpan {
        line,
        start: end,
        end: end.saturating_add(1),
    }
}

// Final conversion from our local line-byte span to the public diagnostic
// range. This is currently byte == character for the ASCII grammar pieces we
// highlight. If Aurora identifiers become Unicode, this is the boundary that
// should change.
fn line_byte_span_to_source_range(span: LineByteSpan) -> SourceRange {
    SourceRange {
        start: SourcePosition {
            line: pest_line_to_source_line(span.line),
            character: span.start as u32,
        },
        end: SourcePosition {
            line: pest_line_to_source_line(span.line),
            character: span.end as u32,
        },
    }
}

// Pest spans already carry start/end line-column positions. Convert those
// directly without token expansion.
fn pest_position_to_source_position(position: LineColumn) -> SourcePosition {
    SourcePosition {
        line: pest_line_to_source_line(position.line),
        character: pest_column_to_byte_index(position.column) as u32,
    }
}

// Pest is 1-based; LSP-style diagnostics are 0-based.
fn pest_line_to_source_line(line: usize) -> u32 {
    line.saturating_sub(1) as u32
}

// Pest columns are 1-based. For our current ASCII grammar, the 0-based column
// is also the byte index in `line_text.as_bytes()`.
fn pest_column_to_byte_index(column: usize) -> usize {
    column.saturating_sub(1)
}

// Makes `LineColLocation` tuples readable at call sites.
impl From<(usize, usize)> for LineColumn {
    fn from((line, column): (usize, usize)) -> Self {
        Self { line, column }
    }
}
