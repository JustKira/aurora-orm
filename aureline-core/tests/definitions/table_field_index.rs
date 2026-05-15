use aureline_core::emit::emit_schema;
use aureline_test_support::{aureline_schema, expected_surql, parse_schema};

use super::assert_validation_contains;

#[test]
fn emits_field_index_and_unique_indexes() {
    let schema = parse_schema(aureline_schema!(
        "table User {",
        "  email string @index",
        "  handle string @unique",
        "}",
    ));

    assert_eq!(
        emit_schema(&schema),
        expected_surql!(
            "DEFINE TABLE user;",
            "DEFINE FIELD email ON user TYPE string;",
            "DEFINE FIELD handle ON user TYPE string;",
            "DEFINE INDEX user_email_idx ON user FIELDS email;",
            "DEFINE INDEX user_handle_unique ON user FIELDS handle UNIQUE;",
        )
    );
}

#[test]
fn duplicate_index_names_are_rejected_within_a_table() {
    assert_validation_contains(
        aureline_schema!(
            "table User {",
            "  email string @index(name: user_lookup)",
            "  handle string @unique(name: user_lookup)",
            "}",
        ),
        "duplicate index name `user_lookup` on table User",
    );
}
