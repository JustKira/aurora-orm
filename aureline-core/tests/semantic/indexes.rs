// Index tests cover semantic lowering for the common non-specialized index
// forms: field-level `@index`/`@unique`, table-level `@@index`/`@@unique`, and
// table-level `@@count`.

use aureline_core::ast::IndexKind;
use aureline_test_support::aureline_schema;

use super::common::{
    assert_no_semantic_errors, assert_semantic_error_contains, checked_schema, table,
};

#[test]
fn field_index_and_unique_lower_to_indexes() {
    let schema = checked_schema(aureline_schema!(
        "table User {",
        "  email string @index",
        "  handle string @unique",
        "}",
    ));
    let table = table(&schema, "User");

    assert_eq!(table.indexes.len(), 2);
    assert!(table.indexes.iter().any(|index| {
        index.name == "user_email_idx"
            && index.fields == vec!["email"]
            && matches!(index.kind, IndexKind::Standard)
    }));
    assert!(table.indexes.iter().any(|index| {
        index.name == "user_handle_unique"
            && index.fields == vec!["handle"]
            && matches!(index.kind, IndexKind::Unique)
    }));
}

#[test]
fn field_index_and_unique_accept_name_keyword() {
    let schema = checked_schema(aureline_schema!(
        "table User {",
        "  email string @unique(name: user_email_lookup)",
        "  status string @index(name: \"idx_user_status\")",
        "}",
    ));
    let table = table(&schema, "User");
    let names: Vec<&str> = table
        .indexes
        .iter()
        .map(|index| index.name.as_str())
        .collect();

    assert!(names.contains(&"user_email_lookup"));
    assert!(names.contains(&"idx_user_status"));
}

#[test]
fn field_index_and_unique_reject_unmodeled_arguments() {
    // Compound indexes live at table level. Field-level `@unique(fields: ...)`
    // would duplicate that concept and make migration semantics ambiguous.
    assert_semantic_error_contains(
        aureline_schema!(
            "table Membership {",
            "  account string",
            "  user string",
            "  account_user string @unique(fields: [account, user])",
            "}",
        ),
        "unknown @unique arg `fields`",
    );
    assert_semantic_error_contains(
        aureline_schema!("table User {", "  email string @index(email)", "}",),
        "@index does not accept positional arguments",
    );
}

#[test]
fn compound_index_and_unique_lower_to_indexes() {
    let schema = checked_schema(aureline_schema!(
        "table User {",
        "  tenant string",
        "  email string",
        "  handle string",
        "",
        "  @@index(fields: [tenant, email], name: tenant_email_lookup)",
        "  @@unique(fields: [tenant, handle])",
        "}",
    ));
    let table = table(&schema, "User");

    assert!(table.indexes.iter().any(|index| {
        index.name == "tenant_email_lookup"
            && index.fields == vec!["tenant", "email"]
            && matches!(index.kind, IndexKind::Standard)
    }));
    assert!(table.indexes.iter().any(|index| {
        index.name == "user_tenant_handle_unique"
            && index.fields == vec!["tenant", "handle"]
            && matches!(index.kind, IndexKind::Unique)
    }));
}

#[test]
fn compound_index_fields_must_be_declared_non_empty_and_unique() {
    // Composite index definitions must resolve to a real ordered field list.
    // Empty, duplicate, or unknown field names are not useful migration input.
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string",
            "",
            "  @@index(fields: [])",
            "}",
        ),
        "requires at least one field",
    );
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string",
            "",
            "  @@unique(fields: [email, email])",
            "}",
        ),
        "duplicate field `email`",
    );
    assert_semantic_error_contains(
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
    let schema = checked_schema(aureline_schema!(
        "table User {",
        "  email string",
        "",
        "  @@count",
        "}",
    ));
    let table = table(&schema, "User");
    let count = table
        .indexes
        .iter()
        .find(|index| matches!(index.kind, IndexKind::Count))
        .expect("count index should exist");

    assert!(count.fields.is_empty());
    assert_eq!(count.name, "user_count");
    assert_semantic_error_contains(
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

#[test]
fn duplicate_index_names_are_rejected_within_one_table() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string @index(name: user_lookup)",
            "  handle string @unique(name: user_lookup)",
            "}",
        ),
        "duplicate index name `user_lookup` on table User",
    );
}

#[test]
fn same_index_name_can_be_reused_on_different_tables() {
    // SurrealDB index names are scoped to a table, so global uniqueness would
    // be stricter than the target database requires.
    assert_no_semantic_errors(aureline_schema!(
        "table User {",
        "  email string @index(name: lookup)",
        "}",
        "",
        "table Organization {",
        "  email string @index(name: lookup)",
        "}",
    ));
}
