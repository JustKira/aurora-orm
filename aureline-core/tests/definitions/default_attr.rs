use aureline_core::emit::emit_schema;
use aureline_test_support::{expected_surql, parse_schema, validation_errors};

#[test]
fn emits_string_number_bool_ident_and_surql_defaults() {
    let schema = parse_schema(aureline_test_support::aureline_schema!(
        "table User {",
        "  name string @default(\"Anonymous\")",
        "  score int @default(10)",
        "  active bool @default(true)",
        "  role string @default(user)",
        "  created_at datetime @default(#surql { time::now() })",
        "}",
    ));

    assert_eq!(
        emit_schema(&schema),
        expected_surql!(
            "DEFINE TABLE user;",
            "DEFINE FIELD active ON user TYPE bool DEFAULT TRUE;",
            "DEFINE FIELD created_at ON user TYPE datetime DEFAULT time::now();",
            "DEFINE FIELD name ON user TYPE string DEFAULT \"Anonymous\";",
            "DEFINE FIELD role ON user TYPE string DEFAULT user;",
            "DEFINE FIELD score ON user TYPE int DEFAULT 10;",
        )
    );
}

#[test]
fn emits_array_and_tuple_defaults() {
    let schema = parse_schema(aureline_test_support::aureline_schema!(
        "table User {",
        "  tags array<string> @default([admin, \"staff\"])",
        "  point object @default((1, 2))",
        "}",
    ));

    assert_eq!(
        emit_schema(&schema),
        expected_surql!(
            "DEFINE TABLE user;",
            "DEFINE FIELD point ON user TYPE object DEFAULT (1, 2);",
            "DEFINE FIELD tags ON user TYPE array<string> DEFAULT [admin, \"staff\"];",
        )
    );
}

#[test]
fn emits_default_always() {
    let schema = parse_schema(aureline_test_support::aureline_schema!(
        "table User {",
        "  updated_at datetime @default(always: true, #surql { time::now() })",
        "}",
    ));

    assert_eq!(
        emit_schema(&schema),
        expected_surql!(
            "DEFINE TABLE user;",
            "DEFINE FIELD updated_at ON user TYPE datetime DEFAULT ALWAYS time::now();",
        )
    );
}

#[test]
fn default_always_can_be_false() {
    let schema = parse_schema(aureline_test_support::aureline_schema!(
        "table User {",
        "  updated_at datetime @default(always: false, #surql { time::now() })",
        "}",
    ));

    assert_eq!(
        emit_schema(&schema),
        expected_surql!(
            "DEFINE TABLE user;",
            "DEFINE FIELD updated_at ON user TYPE datetime DEFAULT time::now();",
        )
    );
}

#[test]
fn default_rejects_missing_value() {
    let errors = validation_errors(aureline_test_support::aureline_schema!(
        "table User {",
        "  name string @default",
        "}",
    ));

    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].message,
        "@default expects exactly one positional value"
    );
}

#[test]
fn default_rejects_keyword_value() {
    let errors = validation_errors(aureline_test_support::aureline_schema!(
        "table User {",
        "  name string @default(value: \"Anonymous\")",
        "}",
    ));

    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].message,
        "unknown @default arg `value`; expected `always`"
    );
}

#[test]
fn default_rejects_non_bool_always() {
    let errors = validation_errors(aureline_test_support::aureline_schema!(
        "table User {",
        "  updated_at datetime @default(always: \"true\", #surql { time::now() })",
        "}",
    ));

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message, "@default `always:` must be a boolean");
}

#[test]
fn default_rejects_multiple_values() {
    let errors = validation_errors(aureline_test_support::aureline_schema!(
        "table User {",
        "  name string @default(\"Anonymous\", \"Guest\")",
        "}",
    ));

    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].message,
        "@default expects exactly one positional value"
    );
}

#[test]
fn default_rejects_duplicate_attributes() {
    let errors = validation_errors(aureline_test_support::aureline_schema!(
        "table User {",
        "  name string @default(\"Anonymous\") @default(\"Guest\")",
        "}",
    ));

    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].message,
        "@default on field `name` is already defined"
    );
}

#[test]
fn default_rejects_invalid_surql_expression() {
    let errors = validation_errors(aureline_test_support::aureline_schema!(
        "table User {",
        "  created_at datetime @default(#surql { SELECT FROM })",
        "}",
    ));

    assert_eq!(errors.len(), 1);
    assert!(
        errors[0].message.contains("SurQL")
            || errors[0].message.contains("parse")
            || errors[0].message.contains("expected"),
        "{}",
        errors[0].message
    );
}

#[test]
fn standalone_always_is_unknown() {
    let errors = validation_errors(aureline_test_support::aureline_schema!(
        "table User {",
        "  updated_at datetime @always",
        "}",
    ));

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message, "unknown field attribute `@always`");
}
