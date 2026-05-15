use aureline_core::surql::{
    parse_query, validate_expression, validate_field_permission, validate_query,
};

#[test]
fn parses_query_through_surrealdb_ast_with_statement_metadata() {
    let parsed = parse_query("LET $c = 'Ada'; RETURN $c;").unwrap();

    assert_eq!(parsed.statement_count(), 2);
    assert_eq!(parsed.let_statements(), ["c"]);
}

#[test]
fn parses_expression_wrapper_through_surrealdb_ast() {
    let parsed = parse_query("RETURN $value != NONE;").unwrap();

    assert_eq!(parsed.statement_count(), 1);
    assert!(parsed.let_statements().is_empty());
}

#[test]
fn query_validation_reports_surrealdb_parse_errors() {
    let error = validate_query("RETURN ;").unwrap_err();

    assert!(error.message.contains("invalid SurrealQL"));
    assert!(error.message.contains("expected"));
}

#[test]
fn validators_accept_valid_single_statement_inputs() {
    validate_query("RETURN 1;").unwrap();
    validate_expression("$value != NONE").unwrap();
    validate_field_permission("SELECT", "WHERE $auth.admin").unwrap();
}

#[test]
fn query_validation_rejects_multiple_statements() {
    let error = validate_query("RETURN 1; RETURN 2;").unwrap_err();

    assert!(error.message.contains("expected exactly one statement"));
    assert!(error.message.contains("found 2"));
}

#[test]
fn query_validation_rejects_let_statements() {
    let error = validate_query("LET $c = 'Ada';").unwrap_err();

    assert!(error.message.contains("LET statements are not allowed"));
    assert!(error.message.contains("c"));
}

#[test]
fn expression_validation_rejects_embedded_statement_terminator() {
    let error = validate_expression("$value != NONE; RETURN true").unwrap_err();

    assert!(error.message.contains("expected exactly one statement"));
}

#[test]
fn expression_validation_rejects_embedded_let_statement() {
    let error = validate_expression("LET $c = 'Ada'; $value != NONE").unwrap_err();

    assert!(error.message.contains("expected exactly one statement"));
}

#[test]
fn field_permission_validation_rejects_embedded_statement_terminator() {
    let error = validate_field_permission("SELECT", "WHERE $auth.admin; RETURN true;").unwrap_err();

    assert!(error.message.contains("expected exactly one statement"));
}

#[test]
fn field_permission_validation_rejects_embedded_let_statement() {
    let error =
        validate_field_permission("SELECT", "WHERE $auth.admin; LET $c = 'Ada'").unwrap_err();

    assert!(error.message.contains("expected exactly one statement"));
}
