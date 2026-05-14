use aureline_core::DiagnosticCode;

use super::common::{
    ExpectedDiagnostic, assert_range, assert_single_diagnostic, diagnostics_for, only_diagnostic,
};

#[test]
fn field_attribute_missing_closing_paren_reports_attribute_call_end() {
    let diagnostics = diagnostics_for(aureline_schema!(
        "table Demo {",
        "  name string @index(name: true",
        "}",
    ));

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected `)` to close attribute arguments",
            start: (1, 31),
            end: (1, 31),
        },
    );
}

#[test]
fn block_attribute_missing_closing_paren_reports_attribute_call_end() {
    let diagnostics = diagnostics_for(aureline_schema!(
        "table Demo {",
        "  @@index(fields: [name]",
        "}",
    ));

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected `)` to close attribute arguments",
            start: (1, 24),
            end: (1, 24),
        },
    );
}

#[test]
fn hnsw_missing_closing_paren_reports_attribute_call_end() {
    let diagnostics = diagnostics_for(aureline_schema!(
        "table Demo {",
        "  v_minimal array<float> @hnsw(dimension: 384",
        "}",
    ));

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected `)` to close attribute arguments",
            start: (1, 45),
            end: (1, 45),
        },
    );
}

#[test]
fn inline_block_attribute_reports_field_line_misuse() {
    let diagnostics = diagnostics_for(aureline_schema!(
        "table Demo {",
        "  role string @@index(fields: [role])",
        "}",
    ));

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "block attributes like `@@index` must be written on their own table line; use `@index` for a field-level index",
            start: (1, 14),
            end: (1, 15),
        },
    );
}

#[test]
fn block_attribute_marker_inside_comment_is_ignored() {
    let diagnostics = diagnostics_for(aureline_schema!(
        "// Block-level annotations (@@) explain composite indexes.",
        "table Demo {",
        "  role string",
        "  @@index(fields: [role])",
        "}",
    ));

    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn assert_surql_body_reports_surrealdb_parse_errors() {
    let diagnostics = diagnostics_for(aureline_schema!(
        "table Demo {",
        "  email string @assert(#surql { $value != })",
        "}",
    ));

    let diagnostic = only_diagnostic(&diagnostics);
    assert_eq!(diagnostic.code, DiagnosticCode::ValidationError);
    assert!(
        diagnostic
            .message
            .starts_with("invalid SurrealQL: Parse error:"),
        "{}",
        diagnostic.message
    );
    assert!(
        diagnostic.message.contains("expected"),
        "{}",
        diagnostic.message
    );
    assert!(
        diagnostic
            .message
            .contains("for example `WHERE $value != NONE`"),
        "{}",
        diagnostic.message
    );
    assert_range(diagnostic, (1, 23), (1, 43));
}

#[test]
fn allow_surql_permission_is_a_known_field_attribute() {
    let diagnostics = diagnostics_for(aureline_schema!(
        "table Demo {",
        "  id string @allow(op: \"SELECT\", #surql { WHERE $value != NONE })",
        "}",
    ));

    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn allow_operation_must_be_a_key_value_arg() {
    let diagnostics = diagnostics_for(aureline_schema!(
        "table Demo {",
        "  id string @allow(SELECT, #surql { WHERE $value != NONE })",
        "}",
    ));

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ValidationError,
            message: "@allow positional arguments must be `#surql { ... }`; use `op: \"SELECT\"` for the operation",
            start: (1, 12),
            end: (1, 59),
        },
    );
}

#[test]
fn allow_op_must_be_a_string_literal() {
    let diagnostics = diagnostics_for(aureline_schema!(
        "table Demo {",
        "  id string @allow(op: RUN, #surql { WHERE $value != NONE })",
        "}",
    ));

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ValidationError,
            message: "@allow `op:` must be a string literal like \"SELECT\"",
            start: (1, 12),
            end: (1, 60),
        },
    );
}

#[test]
fn allow_rejects_unknown_operation() {
    let diagnostics = diagnostics_for(aureline_schema!(
        "table Demo {",
        "  id string @allow(op: \"RUN\", #surql { WHERE $value != NONE })",
        "}",
    ));

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ValidationError,
            message: "unknown @allow operation `RUN`; expected one of: SELECT, CREATE, UPDATE, DELETE",
            start: (1, 12),
            end: (1, 62),
        },
    );
}

#[test]
fn allow_surql_permission_reports_surrealdb_parse_errors() {
    let diagnostics = diagnostics_for(aureline_schema!(
        "table Demo {",
        "  id string @allow(op: \"SELECT\", #surql { WHERE $value != })",
        "}",
    ));

    let diagnostic = only_diagnostic(&diagnostics);
    assert_eq!(diagnostic.code, DiagnosticCode::ValidationError);
    assert!(
        diagnostic
            .message
            .contains("Unexpected end of file, expected an expression"),
        "{}",
        diagnostic.message
    );
    assert!(
        diagnostic
            .message
            .contains("for example `WHERE $value != NONE`"),
        "{}",
        diagnostic.message
    );
    assert!(
        diagnostic.message.contains("expected"),
        "{}",
        diagnostic.message
    );
    assert_range(diagnostic, (1, 33), (1, 59));
}
