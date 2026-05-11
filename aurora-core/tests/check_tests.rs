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
fn check_returns_parse_diagnostic_with_partial_schema_for_recovered_source_item_syntax() {
    let report = check("tabl user { name string }");

    assert!(report.has_errors());
    assert!(report.schema.is_some());
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
fn check_preserves_valid_items_around_recovered_source_item_syntax() {
    let report = check(
        r#"
table user {
  name string
}

tabl post schemafull

table comment {
  body string
}
"#,
    );

    assert!(report.has_errors());
    let schema = report
        .schema
        .as_ref()
        .expect("partial schema should be available");
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(schema.items.len(), 2);
    assert!(
        report.diagnostics[0]
            .message
            .contains("did you mean `table`")
    );
}

#[test]
fn check_does_not_recover_malformed_known_source_declarations() {
    let report = check("table primitives_demo schemafull ");

    assert!(report.has_errors());
    assert!(report.schema.is_none());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.diagnostics[0].message,
        "expected `{` to start table body"
    );
}

#[test]
fn check_does_not_suggest_identical_source_keyword() {
    let report = check("table");

    assert!(report.has_errors());
    assert!(report.schema.is_none());
    assert_eq!(report.diagnostics.len(), 1);
    assert!(
        !report.diagnostics[0]
            .message
            .contains("did you mean `table`")
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
