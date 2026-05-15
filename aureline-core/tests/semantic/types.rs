// Type-reference tests cover semantic checks that require a global schema view.
// The parser can parse `record<User>` without knowing whether `User` exists;
// the semantic layer must resolve that reference.

use aureline_test_support::aureline_schema;

use super::common::{assert_no_semantic_errors, assert_semantic_error_contains};

#[test]
fn unconstrained_record_type_is_valid() {
    // Bare `record` means any record ID and does not reference a specific
    // table symbol.
    assert_no_semantic_errors(aureline_schema!("table Activity {", "  target record", "}",));
}

#[test]
fn record_type_can_reference_declared_table() {
    assert_no_semantic_errors(aureline_schema!(
        "table User {",
        "  id string",
        "}",
        "",
        "table Post {",
        "  author record<User>",
        "}",
    ));
}

#[test]
fn record_type_can_reference_declared_table_after_emit_normalization() {
    // References should resolve against the emitted SurrealDB table name, so
    // `record<User>` can target a declared `table user`.
    assert_no_semantic_errors(aureline_schema!(
        "table user {",
        "  id string",
        "}",
        "",
        "table Post {",
        "  author record<User>",
        "}",
    ));
}

#[test]
fn record_type_can_reference_its_own_table() {
    assert_no_semantic_errors(aureline_schema!(
        "table Category {",
        "  parent record<Category>?",
        "}",
    ));
}

#[test]
fn record_type_rejects_unknown_table_reference() {
    assert_semantic_error_contains(
        aureline_schema!("table Post {", "  author record<User>", "}",),
        "unknown record table `User`",
    );
}

#[test]
fn nested_record_type_rejects_unknown_table_reference() {
    // Record references can hide inside compound types; semantic validation
    // must walk the whole type tree.
    assert_semantic_error_contains(
        aureline_schema!("table Post {", "  reviewers array<record<User>>", "}",),
        "unknown record table `User`",
    );
    assert_semantic_error_contains(
        aureline_schema!("table Post {", "  reviewers set<record<User>>", "}",),
        "unknown record table `User`",
    );
}

#[test]
fn function_signature_record_types_reject_unknown_table_references() {
    let schema = aureline_schema!(
        "function load_owner(owner: record<MissingInput>) -> record<MissingOutput> {",
        "  #surql {",
        "    RETURN $owner;",
        "  }",
        "}",
    );

    assert_semantic_error_contains(&schema, "unknown record table `MissingInput`");
    assert_semantic_error_contains(&schema, "unknown record table `MissingOutput`");
}
