use aureline_core::DiagnosticCode;

use super::common::{
    ExpectedDiagnostic, assert_range, assert_single_diagnostic, diagnostics_for, only_diagnostic,
};

#[test]
fn table_header_without_body_reports_missing_block_open() {
    let diagnostics = diagnostics_for("table primitives_demo schemafull ");

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected `{` to start table body",
            start: (0, 33),
            end: (0, 33),
        },
    );
}

#[test]
fn table_missing_name_reports_identifier() {
    let diagnostics = diagnostics_for("table { }");

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected identifier",
            start: (0, 6),
            end: (0, 7),
        },
    );
}

#[test]
fn typoed_table_keyword_uses_source_recovery_suggestion() {
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

#[test]
fn typoed_schemafull_modifier_highlights_full_word() {
    let diagnostics = diagnostics_for("table primitives_demo schemaful {\n}\n");
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
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
fn typoed_drop_modifier_highlights_full_word() {
    let diagnostics = diagnostics_for("table primitives_demo dorp {\n}\n");
    let diagnostic = only_diagnostic(&diagnostics);

    assert_eq!(diagnostic.code, DiagnosticCode::ParseError);
    assert!(
        diagnostic
            .message
            .contains("`schemafull`, `schemaless`, or `drop`"),
        "{}",
        diagnostic.message
    );
    assert_range(diagnostic, (0, 22), (0, 26));
}
