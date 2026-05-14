// Attribute tests cover generic field/block attribute semantics before the
// specialized index modules take over. These tests focus on argument shape,
// suggestions, and non-index attributes like `@flexible`, `@assert`, and
// `@allow`.

use aureline_test_support::aureline_schema;

use super::common::{
    assert_no_semantic_errors, assert_semantic_error_contains, checked_schema, semantic_errors,
    table,
};

#[test]
fn unknown_field_attribute_errors_with_suggestion() {
    // Unknown attributes are semantic errors because the parser intentionally
    // accepts raw `@name(...)` syntax without knowing every supported name.
    let errors = semantic_errors(aureline_schema!(
        "table User {",
        "  email string @uniqu",
        "}",
    ));

    assert!(
        errors[0].message.contains("@uniqu"),
        "{}",
        errors[0].message
    );
    assert_eq!(errors[0].hint.as_deref(), Some("did you mean `@unique`?"));
}

#[test]
fn unknown_block_attribute_errors_with_suggestion() {
    let errors = semantic_errors(aureline_schema!(
        "table User {",
        "  email string",
        "",
        "  @@uniqu(fields: [email])",
        "}",
    ));

    assert!(
        errors[0].message.contains("@@uniqu"),
        "{}",
        errors[0].message
    );
    assert_eq!(errors[0].hint.as_deref(), Some("did you mean `@@unique`?"));
}

#[test]
fn flexible_on_object_sets_field_flag() {
    let schema = checked_schema(aureline_schema!(
        "table Doc {",
        "  meta object @flexible",
        "}",
    ));

    let table = table(&schema, "Doc");
    assert!(table.fields[0].flexible);
}

#[test]
fn flexible_on_non_object_errors() {
    assert_semantic_error_contains(
        aureline_schema!("table Doc {", "  body string @flexible", "}",),
        "requires `object`",
    );
}

#[test]
fn assert_requires_one_surql_block() {
    assert_semantic_error_contains(
        aureline_schema!("table User {", "  email string @assert(true)", "}",),
        "@assert expects exactly one `#surql { ... }` block",
    );
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string @assert(#surql { $value != NONE }, #surql { $value != '' })",
            "}",
        ),
        "@assert expects exactly one `#surql { ... }` block",
    );
}

#[test]
fn allow_requires_operation_and_permission_block() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string @allow(#surql { WHERE $value != NONE })",
            "}",
        ),
        "@allow requires an `op: \"SELECT\"` argument",
    );
    assert_semantic_error_contains(
        aureline_schema!("table User {", "  email string @allow(op: \"SELECT\")", "}",),
        "@allow requires one positional `#surql { ... }` permission block",
    );
}

#[test]
fn allow_rejects_malformed_arguments() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string @allow(op: \"SELECT\", op: \"UPDATE\", #surql { WHERE true })",
            "}",
        ),
        "@allow has duplicate `op:` arguments",
    );
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string @allow(op: \"SELECT\", #surql { WHERE true }, #surql { WHERE false })",
            "}",
        ),
        "@allow has duplicate `#surql { ... }` permission blocks",
    );
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string @allow(op: RUN, #surql { WHERE true })",
            "}",
        ),
        "@allow `op:` must be a string literal",
    );
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string @allow(mode: \"SELECT\", #surql { WHERE true })",
            "}",
        ),
        "unknown @allow arg `mode`; expected `op`",
    );
}

#[test]
fn allow_rejects_unknown_operation() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string @allow(op: \"RUN\", #surql { WHERE true })",
            "}",
        ),
        "unknown @allow operation `RUN`",
    );
}

#[test]
fn known_attributes_can_be_combined_on_one_table() {
    // This is a broad sanity check that independent attribute handlers compose
    // instead of accidentally overwriting each other during lowering.
    assert_no_semantic_errors(aureline_schema!(
        "analyzer search {",
        "  tokenizers blank",
        "}",
        "",
        "table Doc {",
        "  meta object @flexible",
        "  body string @fulltext(analyzer: search)",
        "  email string @unique",
        "  status string @index",
        "}",
    ));
}
