use aureline_core::emit::emit_schema;
use aureline_test_support::{aureline_schema, expected_surql, parse_schema};

use super::assert_validation_contains;

#[test]
fn emits_compound_and_count_indexes() {
    let schema = parse_schema(aureline_schema!(
        "table User {",
        "  tenant string",
        "  email string",
        "  handle string",
        "",
        "  @@index(fields: [tenant, email], name: tenant_email_lookup)",
        "  @@unique(fields: [tenant, handle])",
        "  @@count",
        "}",
    ));

    assert_eq!(
        emit_schema(&schema),
        expected_surql!(
            "DEFINE TABLE user;",
            "DEFINE FIELD email ON user TYPE string;",
            "DEFINE FIELD handle ON user TYPE string;",
            "DEFINE FIELD tenant ON user TYPE string;",
            "DEFINE INDEX tenant_email_lookup ON user FIELDS tenant, email;",
            "DEFINE INDEX user_count ON user COUNT;",
            "DEFINE INDEX user_tenant_handle_unique ON user FIELDS tenant, handle UNIQUE;",
        )
    );
}

#[test]
fn compound_index_fields_must_exist_be_non_empty_and_unique() {
    assert_validation_contains(
        aureline_schema!(
            "table User {",
            "  email string",
            "",
            "  @@index(fields: [])",
            "}",
        ),
        "at least one field",
    );
    assert_validation_contains(
        aureline_schema!(
            "table User {",
            "  email string",
            "",
            "  @@unique(fields: [email, email])",
            "}",
        ),
        "duplicate field `email`",
    );
    assert_validation_contains(
        aureline_schema!(
            "table User {",
            "  email string",
            "",
            "  @@index(fields: [missing])",
            "}",
        ),
        "unknown field `missing`",
    );
}

#[test]
fn count_index_is_table_level_only() {
    assert_validation_contains(
        aureline_schema!(
            "table User {",
            "  email string",
            "",
            "  @@count(fields: [email])",
            "}",
        ),
        "@@count on User takes no arguments",
    );
}
