// SurQL semantic tests cover two different responsibilities:
// syntax validation delegated to SurrealDB's parser, and Aureline-owned
// variable-scope rules for each escape-hatch context.

use aureline_test_support::aureline_schema;

use super::common::{assert_no_semantic_errors, assert_semantic_error_contains};

#[test]
fn assert_surql_syntax_is_validated() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string @assert(#surql { $value != })",
            "}",
        ),
        "invalid SurrealQL",
    );
}

#[test]
fn allow_surql_syntax_is_validated() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string @allow(op: \"SELECT\", #surql { WHERE $value != })",
            "}",
        ),
        "invalid SurrealQL",
    );
}

#[test]
fn top_level_surql_block_syntax_is_validated() {
    // Top-level raw SurQL is not attached to a field attribute, but it still
    // needs syntax validation before a checked schema is accepted.
    assert_semantic_error_contains(
        aureline_schema!("#surql {", "  RETURN ;", "}",),
        "invalid SurrealQL",
    );
}

#[test]
fn assert_surql_context_allows_value_variable() {
    assert_no_semantic_errors(aureline_schema!(
        "table User {",
        "  email string @assert(#surql { $value != NONE })",
        "}",
    ));
}

#[test]
fn assert_surql_context_rejects_unknown_variable() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string @assert(#surql { $hello != NONE })",
            "}",
        ),
        "unknown SurrealQL variable `$hello`",
    );
}

#[test]
fn assert_surql_context_rejects_permission_only_variables() {
    // `@assert` evaluates a field value expression. Permission-only variables
    // like `$auth` should not leak into that context.
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string @assert(#surql { $auth.id != NONE })",
            "}",
        ),
        "unknown SurrealQL variable `$auth`",
    );
}

#[test]
fn allow_surql_context_allows_auth_and_value_variables() {
    // Permissions can inspect auth state and the field value. Unknown variables
    // are tested separately so the allowed set stays explicit.
    assert_no_semantic_errors(aureline_schema!(
        "table User {",
        "  email string @allow(op: \"SELECT\", #surql { WHERE $auth.id != NONE AND $value != NONE })",
        "}",
    ));
}

#[test]
fn allow_surql_context_rejects_unknown_variable() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string @allow(op: \"SELECT\", #surql { WHERE $hello != NONE })",
            "}",
        ),
        "unknown SurrealQL variable `$hello`",
    );
}
