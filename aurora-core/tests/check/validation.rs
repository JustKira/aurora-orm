use aurora_core::{DiagnosticCode, Severity, check};

#[test]
fn validation_diagnostics_keep_schema_for_lsp_consumers() {
    let report = check(
        r#"
table doc {
  body string @flexible
}
"#,
    );
    let diagnostic = &report.diagnostics[0];

    assert!(report.has_errors());
    assert!(report.schema.is_some());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(diagnostic.code, DiagnosticCode::ValidationError);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert!(
        diagnostic.message.contains("requires `object`"),
        "{}",
        diagnostic.message
    );
    assert_eq!(diagnostic.range.start.line, 2);
    assert_eq!(diagnostic.range.start.character, 14);
    assert_eq!(diagnostic.range.end.line, 2);
    assert_eq!(diagnostic.range.end.character, 23);
}

#[test]
fn recovery_and_validation_diagnostics_can_be_returned_together() {
    let report = check(
        r#"
table doc {
  body string @flexible
}

tabl ignored
"#,
    );

    assert!(report.has_errors());
    assert!(report.schema.is_some());
    assert_eq!(report.diagnostics.len(), 2);
    assert_eq!(report.diagnostics[0].code, DiagnosticCode::ParseError);
    assert_eq!(report.diagnostics[1].code, DiagnosticCode::ValidationError);
    assert!(
        report.diagnostics[0]
            .message
            .contains("unknown source item")
    );
    assert!(report.diagnostics[1].message.contains("requires `object`"));
}
