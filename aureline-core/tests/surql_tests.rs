use aureline_core::surql::{parse_query, validate_query};

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
