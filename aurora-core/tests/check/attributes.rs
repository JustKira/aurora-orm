use aurora_core::DiagnosticCode;

use super::common::{ExpectedDiagnostic, assert_single_diagnostic, diagnostics_for};

#[test]
fn field_attribute_missing_closing_paren_reports_attribute_call_end() {
    let diagnostics = diagnostics_for(
        r#"
table Demo {
  name string @index(name: true
}
"#,
    );

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected `)` to close attribute arguments",
            start: (2, 31),
            end: (2, 31),
        },
    );
}

#[test]
fn block_attribute_missing_closing_paren_reports_attribute_call_end() {
    let diagnostics = diagnostics_for(
        r#"
table Demo {
  @@index(fields: [name]
}
"#,
    );

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected `)` to close attribute arguments",
            start: (2, 24),
            end: (2, 24),
        },
    );
}

#[test]
fn hnsw_missing_closing_paren_reports_attribute_call_end() {
    let diagnostics = diagnostics_for(
        r#"
table Demo {
  v_minimal array<float> @hnsw(dimension: 384
}
"#,
    );

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "expected `)` to close attribute arguments",
            start: (2, 45),
            end: (2, 45),
        },
    );
}

#[test]
fn inline_block_attribute_reports_field_line_misuse() {
    let diagnostics = diagnostics_for(
        r#"
table Demo {
  role string @@index(fields: [role])
}
"#,
    );

    assert_single_diagnostic(
        &diagnostics,
        ExpectedDiagnostic {
            code: DiagnosticCode::ParseError,
            message: "block attributes like `@@index` must be written on their own table line; use `@index` for a field-level index",
            start: (2, 14),
            end: (2, 15),
        },
    );
}

#[test]
fn block_attribute_marker_inside_comment_is_ignored() {
    let diagnostics = diagnostics_for(
        r#"
// Block-level annotations (@@) explain composite indexes.
table Demo {
  role string
  @@index(fields: [role])
}
"#,
    );

    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}
