// Symbol tests define the uniqueness rules for schema-level names.
// These are semantic rules because the grammar can parse duplicate names, but
// a checked schema cannot safely emit or migrate them.

use aureline_test_support::aureline_schema;

use super::common::{assert_no_semantic_errors, assert_semantic_error_contains};

#[test]
fn duplicate_table_names_are_rejected() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string",
            "}",
            "",
            "table User {",
            "  handle string",
            "}",
        ),
        "duplicate table name `User`",
    );
}

#[test]
fn duplicate_table_names_after_emit_normalization_are_rejected() {
    // `User` and `user` both emit to the same SurrealDB table name, so this
    // must be rejected before migration planning.
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string",
            "}",
            "",
            "table user {",
            "  handle string",
            "}",
        ),
        "duplicate table name `user` after normalization",
    );
}

#[test]
fn duplicate_field_names_are_rejected_within_a_table() {
    assert_semantic_error_contains(
        aureline_schema!("table User {", "  email string", "  email int", "}",),
        "duplicate field name `email` on table User",
    );
}

#[test]
fn same_field_name_can_exist_on_different_tables() {
    // Field names are scoped by table. Only duplicates within the same table
    // are invalid.
    assert_no_semantic_errors(aureline_schema!(
        "table User {",
        "  id string",
        "}",
        "",
        "table Organization {",
        "  id string",
        "}",
    ));
}

#[test]
fn duplicate_analyzer_names_are_rejected() {
    assert_semantic_error_contains(
        aureline_schema!(
            "analyzer search {",
            "  tokenizers blank",
            "}",
            "",
            "analyzer search {",
            "  tokenizers class",
            "}",
        ),
        "duplicate analyzer name `search`",
    );
}
