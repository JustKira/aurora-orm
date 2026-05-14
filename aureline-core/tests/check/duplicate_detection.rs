use aureline_core::{DiagnosticCode, check};

use super::common::{assert_range, only_diagnostic};

#[test]
fn duplicate_table_name_points_to_duplicate_declaration() {
    let report = check(aureline_schema!(
        "table User {",
        "  email string",
        "}",
        "",
        "table User {",
        "  handle string",
        "}",
    ));

    let diagnostic = only_diagnostic(&report.diagnostics);
    assert_eq!(diagnostic.code, DiagnosticCode::ValidationError);
    assert!(diagnostic.message.contains("duplicate table name `User`"));
    assert_range(diagnostic, (4, 6), (4, 10));
}

#[test]
fn normalized_duplicate_table_name_points_to_duplicate_declaration() {
    let report = check(aureline_schema!(
        "table User {",
        "  email string",
        "}",
        "",
        "table user {",
        "  handle string",
        "}",
    ));

    let diagnostic = only_diagnostic(&report.diagnostics);
    assert_eq!(diagnostic.code, DiagnosticCode::ValidationError);
    assert!(
        diagnostic
            .message
            .contains("duplicate table name `user` after normalization")
    );
    assert_range(diagnostic, (4, 6), (4, 10));
}

#[test]
fn duplicate_field_name_points_to_duplicate_declaration() {
    let report = check(aureline_schema!(
        "table User {",
        "  email string",
        "  email int",
        "}",
    ));

    let diagnostic = only_diagnostic(&report.diagnostics);
    assert_eq!(diagnostic.code, DiagnosticCode::ValidationError);
    assert!(
        diagnostic
            .message
            .contains("duplicate field name `email` on table User")
    );
    assert_range(diagnostic, (2, 2), (2, 7));
}

#[test]
fn duplicate_analyzer_name_points_to_duplicate_declaration() {
    let report = check(aureline_schema!(
        "analyzer search {",
        "  tokenizers blank",
        "}",
        "",
        "analyzer search {",
        "  tokenizers class",
        "}",
    ));

    let diagnostic = only_diagnostic(&report.diagnostics);
    assert_eq!(diagnostic.code, DiagnosticCode::ValidationError);
    assert!(
        diagnostic
            .message
            .contains("duplicate analyzer name `search`")
    );
    assert_range(diagnostic, (4, 9), (4, 15));
}
