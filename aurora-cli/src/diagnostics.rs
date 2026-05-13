use std::ops::Range;
use std::path::Path;

use anyhow::Result;
use aurora_core::{Diagnostic, Severity, SourcePosition, SourceRange};
use codespan_reporting::diagnostic::{
    Diagnostic as CodespanDiagnostic, Label, Severity as CodespanSeverity,
};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term::termcolor::WriteColor;
use codespan_reporting::term::{self, Chars, Config};

pub(crate) fn emit_diagnostic(
    writer: &mut dyn WriteColor,
    path: &Path,
    source: &str,
    diagnostic: &Diagnostic,
) -> Result<()> {
    let file_name = path.display().to_string();
    let file = SimpleFile::new(file_name, source);
    let diagnostic = to_codespan_diagnostic(diagnostic, source);
    let config = Config {
        chars: Chars::ascii(),
        ..Config::default()
    };

    term::emit(writer, &config, &file, &diagnostic)?;
    Ok(())
}

fn to_codespan_diagnostic(diagnostic: &Diagnostic, source: &str) -> CodespanDiagnostic<()> {
    CodespanDiagnostic::new(to_codespan_severity(diagnostic.severity))
        .with_code(diagnostic.code.as_str())
        .with_message(&diagnostic.message)
        .with_label(Label::primary(
            (),
            source_range_to_byte_range(source, diagnostic.range),
        ))
        .with_notes(diagnostic.help.clone())
}

fn to_codespan_severity(severity: Severity) -> CodespanSeverity {
    match severity {
        Severity::Error => CodespanSeverity::Error,
        Severity::Warning => CodespanSeverity::Warning,
        Severity::Info => CodespanSeverity::Note,
        Severity::Hint => CodespanSeverity::Help,
    }
}

fn source_range_to_byte_range(source: &str, range: SourceRange) -> Range<usize> {
    let start = position_to_byte_offset(source, range.start);
    let mut end = position_to_byte_offset(source, range.end);
    if end <= start {
        end = next_char_boundary(source, start);
    }
    start..end
}

fn position_to_byte_offset(source: &str, position: SourcePosition) -> usize {
    let mut line_start = 0;
    for (line_index, line) in source.split_inclusive('\n').enumerate() {
        if line_index == position.line as usize {
            let line_end = line_start + line.trim_end_matches(['\r', '\n']).len();
            return line_start
                + character_to_byte_offset(&source[line_start..line_end], position.character);
        }
        line_start += line.len();
    }
    source.len()
}

fn character_to_byte_offset(line: &str, character: u32) -> usize {
    line.char_indices()
        .map(|(idx, _)| idx)
        .chain(std::iter::once(line.len()))
        .nth(character as usize)
        .unwrap_or(line.len())
}

fn next_char_boundary(source: &str, offset: usize) -> usize {
    if offset >= source.len() {
        return source.len();
    }
    source[offset..]
        .char_indices()
        .nth(1)
        .map_or(source.len(), |(idx, _)| offset + idx)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use aurora_core::{Diagnostic, DiagnosticCode, Severity, SourcePosition, SourceRange};
    use codespan_reporting::term::termcolor::Buffer;

    use super::{emit_diagnostic, source_range_to_byte_range};

    #[test]
    fn maps_lsp_range_to_utf8_byte_range() {
        let source = "table Demo {\n  café string @foo\n}\n";
        let range = SourceRange {
            start: SourcePosition {
                line: 1,
                character: 14,
            },
            end: SourcePosition {
                line: 1,
                character: 18,
            },
        };

        assert_eq!(&source[source_range_to_byte_range(source, range)], "@foo");
    }

    #[test]
    fn renders_diagnostic_with_source_snippet() {
        let source = "table Demo {\n  email string @foo\n}\n";
        let diagnostic = Diagnostic {
            severity: Severity::Error,
            code: DiagnosticCode::ValidationError,
            message: "unknown field attribute `@foo`".to_string(),
            range: SourceRange {
                start: SourcePosition {
                    line: 1,
                    character: 15,
                },
                end: SourcePosition {
                    line: 1,
                    character: 19,
                },
            },
            help: vec!["did you mean `@hnsw`?".to_string()],
            data: None,
        };
        let mut writer = Buffer::no_color();

        emit_diagnostic(&mut writer, Path::new("schema.aurora"), source, &diagnostic).unwrap();

        let rendered = String::from_utf8(writer.into_inner()).unwrap();
        assert!(rendered.contains("error[validation_error]: unknown field attribute `@foo`"));
        assert!(rendered.contains("--> schema.aurora:2:16"));
        assert!(rendered.contains("2 |   email string @foo"));
        assert!(rendered.contains("did you mean `@hnsw`?"));
    }
}
