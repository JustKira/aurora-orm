use aurora_core::{Diagnostic, DiagnosticCode, Severity, check};

#[test]
fn malformed_table_header_reports_missing_block_open_at_insertion_point() {
    let diagnostics = diagnostics_for("table primitives_demo schemafull ");

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected `{` to start table body",
            start: (0, 33),
            end: (0, 34),
        },
    );
}

#[test]
fn malformed_analyzer_header_reports_missing_block_open_at_insertion_point() {
    let diagnostics = diagnostics_for("analyzer edu ");

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected `{` to start analyzer body",
            start: (0, 13),
            end: (0, 14),
        },
    );
}

#[test]
fn typoed_table_modifier_highlights_full_word() {
    let diagnostics = diagnostics_for("table primitives_demo schemaful { }");
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert!(
        diagnostic
            .message
            .contains("`schemafull`, `schemaless`, or `drop`"),
        "{}",
        diagnostic.message
    );
    assert_range(diagnostic, (0, 22), (0, 31));
}

#[test]
fn invalid_type_highlights_full_token_for_lsp_consumers() {
    let diagnostics = diagnostics_for("table Demo { ttl duratio }");
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert!(
        diagnostic.message.contains("type"),
        "{}",
        diagnostic.message
    );
    assert_range(diagnostic, (0, 17), (0, 24));
}

#[test]
fn array_missing_length_explains_array_syntax() {
    let diagnostics = diagnostics_for("table Demo { tags array<string, > }");
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
    let diagnostics = diagnostics_for("table Demo { tags set<string, > }");
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
    let diagnostics = diagnostics_for("table Demo { shape geometry<> }");
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
fn record_missing_type_close_documents_current_field_context_error() {
    let diagnostics = diagnostics_for("table Demo { owner record<User }");

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected field or block attribute, `?`, or field attribute",
            start: (0, 25),
            end: (0, 26),
        },
    );
}

#[test]
fn attribute_call_missing_closing_paren_documents_current_value_error() {
    let diagnostics = diagnostics_for("table Demo { name string @index(name: true }");

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected `)` to close attribute arguments",
            start: (0, 44),
            end: (0, 45),
        },
    );
}

#[test]
fn block_attribute_missing_arguments_documents_current_value_error() {
    let diagnostics = diagnostics_for("table Demo { @@index(fields: [name] }");

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected `)` to close attribute arguments",
            start: (0, 37),
            end: (0, 38),
        },
    );
}

#[test]
fn hnsw_missing_closing_paren_reports_attribute_call_end() {
    let diagnostics = diagnostics_for("table Demo { v_minimal array<float> @hnsw(dimension: 384 ");

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected `)` to close attribute arguments",
            start: (0, 57),
            end: (0, 58),
        },
    );
}

#[test]
fn unknown_source_item_diagnostic_suggests_known_keyword() {
    let diagnostics = diagnostics_for("tabl compound_demo schemafull");

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "unknown source item `tabl`; did you mean `table`?",
            start: (0, 0),
            end: (0, 29),
        },
    );
}

#[test]
fn unknown_source_item_without_close_keyword_has_generic_message() {
    let diagnostics = diagnostics_for("nonsense stuff");

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "invalid source item",
            start: (0, 0),
            end: (0, 14),
        },
    );
}

#[test]
fn known_keyword_without_required_parts_does_not_use_recovery_message() {
    let diagnostics = diagnostics_for("table");
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert!(!diagnostic.message.contains("unknown source item"));
    assert!(!diagnostic.message.contains("did you mean `table`"));
}

fn diagnostics_for(source: &str) -> Vec<Diagnostic> {
    let report = check(source);
    eprintln!("source:\n{source}\n");
    for diagnostic in &report.diagnostics {
        eprintln!("diagnostic: {diagnostic:#?}");
    }
    report.diagnostics
}

struct ExpectedDiagnostic<'a> {
    code: DiagnosticCode,
    message: &'a str,
    start: (u32, u32),
    end: (u32, u32),
}

fn assert_single_diagnostic(diagnostics: &[Diagnostic], expected: ExpectedDiagnostic<'_>) {
    let diagnostic = only_diagnostic(diagnostics);
    assert_eq!(diagnostic.code, expected.code);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(diagnostic.message, expected.message);
    assert_range(diagnostic, expected.start, expected.end);
}

fn only_diagnostic(diagnostics: &[Diagnostic]) -> &Diagnostic {
    assert_eq!(
        diagnostics.len(),
        1,
        "expected one diagnostic, got {diagnostics:#?}"
    );
    &diagnostics[0]
}

fn assert_range(diagnostic: &Diagnostic, start: (u32, u32), end: (u32, u32)) {
    assert_eq!(diagnostic.range.start.line, start.0);
    assert_eq!(diagnostic.range.start.character, start.1);
    assert_eq!(diagnostic.range.end.line, end.0);
    assert_eq!(diagnostic.range.end.character, end.1);
}
