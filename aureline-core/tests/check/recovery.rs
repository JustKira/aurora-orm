use aureline_core::{DiagnosticCode, Severity, check};

#[test]
fn valid_schema_returns_schema_without_diagnostics() {
    let report = check(aureline_schema!("table user {", "  name string", "}",));

    assert!(!report.has_errors());
    assert!(report.diagnostics.is_empty());
    assert!(report.schema.is_some());
}

#[test]
fn typoed_source_item_recovers_with_partial_schema() {
    let report = check("tabl user { name string }");
    let diagnostic = &report.diagnostics[0];

    assert!(report.has_errors());
    assert!(report.schema.is_some());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(
        diagnostic.message,
        "unknown source item `tabl`; did you mean `table`?"
    );
    assert_eq!(diagnostic.range.start.line, 0);
    assert_eq!(diagnostic.range.start.character, 0);
    assert_eq!(diagnostic.range.end.line, 0);
    assert_eq!(diagnostic.range.end.character, 25);
}

#[test]
fn recovery_preserves_valid_items_around_invalid_source_item() {
    let report = check(aureline_schema!(
        "table user {",
        "  name string",
        "}",
        "",
        "tabl post schemafull",
        "",
        "table comment {",
        "  body string",
        "}",
    ));

    let schema = report
        .schema
        .as_ref()
        .expect("partial schema should be available");
    let diagnostic = &report.diagnostics[0];

    assert!(report.has_errors());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(schema.items.len(), 2);
    assert_eq!(
        diagnostic.message,
        "unknown source item `tabl`; did you mean `table`?"
    );
    assert_eq!(diagnostic.range.start.line, 4);
    assert_eq!(diagnostic.range.start.character, 0);
    assert_eq!(diagnostic.range.end.line, 5);
    assert_eq!(diagnostic.range.end.character, 0);
}

#[test]
fn recovery_does_not_suggest_identical_source_keyword() {
    let report = check("table");
    let diagnostic = &report.diagnostics[0];

    assert!(report.has_errors());
    assert!(report.schema.is_none());
    assert_eq!(report.diagnostics.len(), 1);
    assert!(!diagnostic.message.contains("did you mean `table`"));
}
