use aureline_core::ast::SchemaItem;
use aureline_test_support::aureline_schema;

use super::common::{assert_semantic_error_contains, checked_schema, semantic_errors};

#[test]
fn function_allow_run_round_trips_through_semantic_validation() {
    let schema = checked_schema(aureline_schema!(
        "function get_full_name(first: string, last: string) -> string {",
        "  #surql {",
        "    RETURN $first + ' ' + $last;",
        "  }",
        "  @@allow(op: \"RUN\", #surql { WHERE owner = $auth.id })",
        "}",
    ));

    let function = schema
        .items
        .iter()
        .find_map(|item| match item {
            SchemaItem::FunctionDecl(function) => Some(function),
            _ => None,
        })
        .expect("function should be present");

    assert_eq!(function.name, "get_full_name");
    assert_eq!(function.raw_attributes.len(), 1);
    assert_eq!(function.raw_attributes[0].name, "allow");
}

#[test]
fn function_rejects_non_allow_block_attribute() {
    assert_semantic_error_contains(
        aureline_schema!(
            "function get_full_name(first: string, last: string) -> string {",
            "  #surql {",
            "    RETURN $first + ' ' + $last;",
            "  }",
            "  @@index(fields: [first])",
            "}",
        ),
        "unknown function block attribute `@@index`",
    );
}

#[test]
fn function_allow_only_accepts_run_operation() {
    assert_semantic_error_contains(
        aureline_schema!(
            "function get_full_name(first: string, last: string) -> string {",
            "  #surql {",
            "    RETURN $first + ' ' + $last;",
            "  }",
            "  @@allow(op: \"SELECT\", #surql { WHERE owner = $auth.id })",
            "}",
        ),
        "expected RUN",
    );
}

#[test]
fn function_allow_rejects_invalid_permission_surql() {
    let errors = semantic_errors(aureline_schema!(
        "function get_full_name(first: string, last: string) -> string {",
        "  #surql {",
        "    RETURN $first + ' ' + $last;",
        "  }",
        "  @@allow(op: \"RUN\", #surql { WHERE $auth.id != })",
        "}",
    ));

    let error = errors
        .iter()
        .find(|error| error.message.contains("invalid SurrealQL"))
        .expect("invalid function permission SurQL should be reported");
    assert!(error.range.is_some(), "{errors:#?}");
}

#[test]
fn function_body_must_reference_declared_arguments() {
    assert_semantic_error_contains(
        aureline_schema!(
            "function get_full_name(first: string, last: string) -> string {",
            "  #surql {",
            "    RETURN $first;",
            "  }",
            "  @@allow(op: \"RUN\", #surql { WHERE owner = $auth.id })",
            "}",
        ),
        "missing references for function arguments: `$last`",
    );
}

#[test]
fn function_body_rejects_unknown_parameters() {
    assert_semantic_error_contains(
        aureline_schema!(
            "function get_full_name(first: string, last: string) -> string {",
            "  #surql {",
            "    RETURN $first + ' ' + $middle + ' ' + $last;",
            "  }",
            "  @@allow(op: \"RUN\", #surql { WHERE owner = $auth.id })",
            "}",
        ),
        "unknown function body parameters: `$middle`",
    );
}

#[test]
fn function_validation_can_report_multiple_errors() {
    let errors = semantic_errors(aureline_schema!(
        "function get_full_name(first: string, last: string) -> string {",
        "  #surql {",
        "    RETURN $first + ' ' + $middle;",
        "  }",
        "  @@allow(op: \"DELETE\", #surql { WHERE true })",
        "}",
    ));

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("missing references")),
        "{errors:#?}"
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("expected RUN")),
        "{errors:#?}"
    );
}
