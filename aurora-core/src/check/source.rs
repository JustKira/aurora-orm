use super::diagnostics::{Diagnostic, DiagnosticCode, SourcePosition, SourceRange};

// Source-shape diagnostics catch layout mistakes that the current Pest grammar
// cannot express cleanly because `WHITESPACE` globally skips newlines. Keep
// this layer narrow: it should flag obvious LSP/CLI ergonomics issues, not
// become a second parser.
pub(super) fn source_shape_diagnostics(source: &str) -> Vec<Diagnostic> {
    inline_block_attribute_diagnostics(source)
}

fn inline_block_attribute_diagnostics(source: &str) -> Vec<Diagnostic> {
    source
        .lines()
        .enumerate()
        .filter_map(|(line_index, line)| inline_block_attribute_diagnostic(line_index, line))
        .collect()
}

fn inline_block_attribute_diagnostic(line_index: usize, line: &str) -> Option<Diagnostic> {
    let block_attr_start = line.find("@@")?;

    // A valid block attribute starts the logical table line, e.g.
    // `@@index(fields: [...])`. If field text appears before `@@`, Pest can
    // currently parse it as `field` + `block_attribute`; this diagnostic makes
    // that shape explicitly invalid for check/LSP consumers.
    if line[..block_attr_start].trim().is_empty() {
        return None;
    }

    let attr_end = block_attr_end(line, block_attr_start);
    Some(Diagnostic::error(
        DiagnosticCode::ParseError,
        "block attributes like `@@index` must be written on their own table line; use `@index` for a field-level index",
        SourceRange {
            start: SourcePosition {
                line: line_index as u32,
                character: block_attr_start as u32,
            },
            end: SourcePosition {
                line: line_index as u32,
                character: attr_end as u32,
            },
        },
    ))
}

fn block_attr_end(line: &str, start: usize) -> usize {
    let bytes = line.as_bytes();
    let mut end = start + 2;
    while end < bytes.len() && is_attr_name_byte(bytes[end]) {
        end += 1;
    }
    end
}

fn is_attr_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}
