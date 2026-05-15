// These tests document the high-level boundary we want:
// raw AST is syntax only, while checked schema is syntax + semantic validation
// + lowering of raw attributes into structured schema data.

use aureline_core::ast::{IndexKind, SchemaItem};
use aureline_test_support::aureline_schema;

use super::common::{assert_semantic_error_contains, checked_schema, raw_ast, table};

#[test]
fn raw_ast_keeps_attributes_uninterpreted_before_semantics() {
    // The parser should preserve user-written attributes exactly as syntax;
    // it should not decide that `@unique` means an index yet.
    let schema = raw_ast(aureline_schema!(
        "table User {",
        "  email string @unique",
        "  meta object @flexible",
        "}",
    ));
    let table = table(&schema, "User");

    assert!(table.indexes.is_empty());
    assert_eq!(table.fields[0].raw_attributes[0].name, "unique");
    assert_eq!(table.fields[1].raw_attributes[0].name, "flexible");
    assert!(!table.fields[1].flexible);
}

#[test]
fn semantic_validation_lowers_attributes_into_checked_schema() {
    // The checked schema is the migration/emitter-facing representation:
    // attributes have been interpreted and copied into structured fields.
    let schema = checked_schema(aureline_schema!(
        "table User {",
        "  email string @unique",
        "  meta object @flexible",
        "}",
    ));
    let table = table(&schema, "User");

    assert!(table.fields[1].flexible);
    assert_eq!(table.indexes.len(), 1);
    assert_eq!(table.indexes[0].name, "user_email_unique");
    assert!(matches!(table.indexes[0].kind, IndexKind::Unique));
}

#[test]
fn parser_success_does_not_mean_semantic_success() {
    // Duplicate tables are syntactically parseable. The semantic layer, not
    // the parser, is responsible for rejecting the schema.
    let schema = raw_ast(aureline_schema!(
        "table User {",
        "  email string",
        "}",
        "",
        "table User {",
        "  handle string",
        "}",
    ));
    assert!(matches!(schema.items[0], SchemaItem::TableDecl(_)));

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
