use aurora_core::{DiagnosticCode, check};

#[test]
fn check_returns_schema_without_diagnostics_for_valid_schema() {
    let report = check(
        r#"
table user {
  name string
}
"#,
    );

    assert!(!report.has_errors());
    assert!(report.diagnostics.is_empty());
    assert!(report.schema.is_some());
}

#[test]
fn check_returns_parse_diagnostic_without_schema_for_invalid_syntax() {
    let report = check("tabl user { name string }");

    assert!(report.has_errors());
    assert!(report.schema.is_none());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].code, DiagnosticCode::ParseError);
    assert!(
        report.diagnostics[0]
            .message
            .contains("did you mean `table`"),
        "{}",
        report.diagnostics[0].message
    );
}

#[test]
fn check_returns_validation_diagnostics_with_schema_for_invalid_semantics() {
    let report = check(
        r#"
table doc {
  body string @flexible
}
"#,
    );

    assert!(report.has_errors());
    assert!(report.schema.is_some());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].code, DiagnosticCode::ValidationError);
    assert!(
        report.diagnostics[0].message.contains("requires `object`"),
        "{}",
        report.diagnostics[0].message
    );
}
