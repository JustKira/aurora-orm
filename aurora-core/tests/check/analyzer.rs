use aurora_core::DiagnosticCode;

use super::common::{
    ExpectedDiagnostic, assert_range, assert_single_diagnostic, diagnostics_for, only_diagnostic,
};

#[test]
fn analyzer_header_without_body_reports_missing_block_open() {
    let diagnostics = diagnostics_for("analyzer edu ");

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected `{` to start analyzer body",
            start: (0, 13),
            end: (0, 13),
        },
    );
}

#[test]
fn analyzer_missing_name_reports_identifier() {
    let diagnostics = diagnostics_for("analyzer { }");

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected identifier",
            start: (0, 9),
            end: (0, 10),
        },
    );
}

#[test]
fn typoed_analyzer_keyword_uses_source_recovery_suggestion() {
    let diagnostics = diagnostics_for("analzyer edu");

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "unknown source item `analzyer`; did you mean `analyzer`?",
            start: (0, 0),
            end: (0, 12),
        },
    );
}

#[test]
fn analyzer_typoed_tokenizers_clause_highlights_word() {
    let diagnostics = diagnostics_for(
        r#"
analyzer edu {
  tokenizer blank
}
"#,
    );
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert!(
        diagnostic.message.contains("analyzer clause"),
        "{}",
        diagnostic.message
    );
    assert_range(diagnostic, (2, 2), (2, 11));
}

#[test]
fn analyzer_typoed_filters_clause_highlights_word() {
    let diagnostics = diagnostics_for(
        r#"
analyzer edu {
  filtres lowercase
}
"#,
    );
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert!(
        diagnostic.message.contains("analyzer clause"),
        "{}",
        diagnostic.message
    );
    assert_range(diagnostic, (2, 2), (2, 9));
}

#[test]
fn analyzer_filter_missing_closing_paren_reports_close_paren() {
    let diagnostics = diagnostics_for(
        r#"
analyzer edu {
  filters lowercase, snowball(english
}
"#,
    );
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert!(
        diagnostic.message.contains("expected"),
        "{}",
        diagnostic.message
    );
    assert_range(diagnostic, (2, 29), (2, 30));
}
