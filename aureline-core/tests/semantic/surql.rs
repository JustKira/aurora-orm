// SurQL semantic tests cover two different responsibilities:
// syntax validation delegated to SurrealDB's parser, and Aureline-owned
// variable-scope rules for each escape-hatch context.

use aureline_test_support::aureline_schema;

use super::common::{assert_no_semantic_errors, assert_semantic_error_contains, semantic_errors};

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
            "  email string @allow(op: \"SELECT\", #surql { WHERE $auth.id != })",
            "}",
        ),
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
fn assert_surql_context_allows_input_variable() {
    assert_no_semantic_errors(aureline_schema!(
        "table User {",
        "  email string @assert(#surql { $input != NONE })",
        "}",
    ));
}

#[test]
fn assert_surql_context_allows_this_variable() {
    assert_no_semantic_errors(aureline_schema!(
        "table User {",
        "  email string @assert(#surql { $this.id != NONE })",
        "}",
    ));
}

#[test]
fn assert_surql_context_allows_closure_parameter() {
    assert_no_semantic_errors(aureline_schema!(
        "table User {",
        "  color array<int> @assert(#surql { $value.all(|$val| $val IN 0..=255) })",
        "}",
    ));
}

#[test]
fn assert_surql_context_allows_separate_closure_parameters() {
    assert_no_semantic_errors(aureline_schema!(
        "table User {",
        "  scores array<int> @assert(#surql { $value.filter(|$val| $val != NONE).all(|$val| $val > 0) })",
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
fn assert_surql_context_rejects_undeclared_closure_variable() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  color array<int> @assert(#surql { $val IN 0..=255 })",
            "}",
        ),
        "unknown SurrealQL variable `$val`",
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
fn semantic_scope_errors_keep_surql_block_range() {
    // Semantic errors currently point at the escape hatch range. We can refine
    // this later by carrying the SurQL body start offset through the AST.
    let errors = semantic_errors(aureline_schema!(
        "table User {",
        "  email string @assert(#surql { $hello != NONE })",
        "}",
    ));
    let error = errors
        .iter()
        .find(|error| {
            error
                .message()
                .contains("unknown SurrealQL variable `$hello`")
        })
        .expect("expected unknown variable error");
    let range = error.range.expect("semantic error should keep a range");

    assert_eq!(range.start.line, 1);
    assert_eq!(range.start.character, 23);
    assert_eq!(range.end.line, 1);
    assert_eq!(range.end.character, 48);
}

#[test]
fn allow_surql_context_allows_auth_variable() {
    // Permissions are auth-context checks. Field values should be validated
    // through `@assert`, not read from `@allow`.
    assert_no_semantic_errors(aureline_schema!(
        "table User {",
        "  email string @allow(op: \"SELECT\", #surql { WHERE $auth.id != NONE })",
        "}",
    ));
}

#[test]
fn allow_surql_context_rejects_value_variable() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table User {",
            "  email string @allow(op: \"SELECT\", #surql { WHERE $value != NONE })",
            "}",
        ),
        "unknown SurrealQL variable `$value`",
    );
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
