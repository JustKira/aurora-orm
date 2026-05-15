//! SurrealQL validation helpers.
//!
//! Aureline owns the surrounding schema syntax. SurrealDB owns the escaped
//! `#surql` body syntax, so validation delegates to SurrealDB's parser.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurqlParseError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSurql {
    statement_count: usize,
    let_statements: Vec<String>,
}

impl ParsedSurql {
    pub fn statement_count(&self) -> usize {
        self.statement_count
    }

    pub fn let_statements(&self) -> &[String] {
        &self.let_statements
    }
}

pub fn validate_expression(body: &str) -> Result<(), SurqlParseError> {
    validate_query(&format!("RETURN {};", body.trim()))
}

pub fn validate_field_permission(operation: &str, body: &str) -> Result<(), SurqlParseError> {
    // TODO: infer this TYPE from the Aureline field type instead of hardcoding
    // `string`; SurrealDB may validate permission expressions differently
    // depending on the field type.
    validate_query(&format!(
        "DEFINE FIELD __aureline__ ON __aureline__ TYPE string PERMISSIONS FOR {operation} {}",
        body.trim()
    ))
}

pub fn validate_query(query: &str) -> Result<(), SurqlParseError> {
    let parsed = parse_query(query)?;
    validate_single_statement(&parsed)
}

fn validate_single_statement(parsed: &ParsedSurql) -> Result<(), SurqlParseError> {
    if parsed.statement_count() != 1 {
        return Err(SurqlParseError {
            message: format!(
                "invalid SurrealQL: expected exactly one statement, found {}",
                parsed.statement_count()
            ),
        });
    }

    if !parsed.let_statements().is_empty() {
        return Err(SurqlParseError {
            message: format!(
                "invalid SurrealQL: LET statements are not allowed in this context: {}",
                parsed.let_statements().join(", ")
            ),
        });
    }

    Ok(())
}

pub fn parse_query(query: &str) -> Result<ParsedSurql, SurqlParseError> {
    surrealdb_core::syn::parse(query)
        .map(|ast| ParsedSurql {
            statement_count: ast.num_statements(),
            let_statements: ast.get_let_statements(),
        })
        .map_err(|error| SurqlParseError {
            message: format_surql_error(error),
        })
}

fn format_surql_error(error: impl fmt::Display) -> String {
    explain_surql_error(error.to_string())
}

fn explain_surql_error(message: String) -> String {
    let mut rendered = format!("invalid SurrealQL: {message}");
    if let Some(help) = surql_error_help(&message) {
        rendered.push_str("\nhelp: ");
        rendered.push_str(help);
    }
    rendered
}

fn surql_error_help(message: &str) -> Option<&'static str> {
    if message.contains("expected an expression") {
        return Some(
            "write a valid SurrealQL expression; use `$value != NONE` in `@assert` or `WHERE $auth.role = \"admin\"` in `@allow`",
        );
    }
    None
}
