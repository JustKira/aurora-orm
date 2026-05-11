use aurora_core::DiagnosticCode;

use super::common::{
    ExpectedDiagnostic, assert_range, assert_single_diagnostic, diagnostics_for, only_diagnostic,
};

#[test]
fn invalid_type_highlights_full_token_for_lsp_consumers() {
    let diagnostics = diagnostics_for(
        r#"
table Demo {
  ttl duratio
}
"#,
    );
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert!(
        diagnostic.message.contains("type"),
        "{}",
        diagnostic.message
    );
    assert_range(diagnostic, (2, 6), (2, 13));
}

#[test]
fn array_missing_length_explains_array_syntax() {
    let diagnostics = diagnostics_for(
        r#"
table Demo {
  tags array<string, >
}
"#,
    );
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert!(
        diagnostic
            .message
            .contains("array types look like `array<T>` or `array<T, N>`"),
        "{}",
        diagnostic.message
    );
}

#[test]
fn set_missing_length_explains_set_syntax() {
    let diagnostics = diagnostics_for(
        r#"
table Demo {
  tags set<string, >
}
"#,
    );
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert!(
        diagnostic
            .message
            .contains("set types look like `set<T>` or `set<T, N>`"),
        "{}",
        diagnostic.message
    );
}

#[test]
fn geometry_empty_feature_list_explains_expected_features() {
    let diagnostics = diagnostics_for(
        r#"
table Demo {
  shape geometry<>
}
"#,
    );
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert!(
        diagnostic.message.contains("geometry feature name")
            || diagnostic.message.contains("geometry types require"),
        "{}",
        diagnostic.message
    );
}

#[test]
fn non_ascii_text_before_parse_error_does_not_panic() {
    let diagnostics = diagnostics_for(
        r#"
/// مرحبا
table Demo {
  name string @default("x"
}
"#,
    );
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert_eq!(
        diagnostic.message,
        "expected `)` to close attribute arguments"
    );
}

#[test]
fn record_missing_type_close_documents_current_field_context_error() {
    let diagnostics = diagnostics_for(
        r#"
table Demo {
  owner record<User
}
"#,
    );

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected end of file, `?`, or field attribute",
            start: (2, 14),
            end: (2, 15),
        },
    );
}

#[test]
fn missing_field_type_reports_type() {
    let diagnostics = diagnostics_for(
        r#"
table Demo {
  name
}
"#,
    );
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert!(
        diagnostic.message.contains("type"),
        "{}",
        diagnostic.message
    );
    assert_range(diagnostic, (2, 6), (2, 7));
}

#[test]
fn type_written_as_field_name_reports_missing_type() {
    let diagnostics = diagnostics_for(
        r#"
table Demo {
  string
}
"#,
    );
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert!(
        diagnostic.message.contains("type"),
        "{}",
        diagnostic.message
    );
    assert_range(diagnostic, (2, 8), (2, 9));
}
