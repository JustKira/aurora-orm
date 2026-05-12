//! SurrealQL validation helpers.
//!
//! Aurora owns the surrounding schema syntax. SurrealDB owns the escaped
//! `#surql` body syntax, so validation delegates to SurrealDB's parser.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurqlParseError {
    pub message: String,
}

pub fn validate_expression(body: &str) -> Result<(), SurqlParseError> {
    surrealdb_core::syn::value(body.trim())
        .map(|_| ())
        .map_err(|error| SurqlParseError {
            message: format_surql_error(error),
        })
}

pub fn validate_field_permission(operation: &str, body: &str) -> Result<(), SurqlParseError> {
    // TODO: infer this TYPE from the Aurora field type instead of hardcoding
    // `string`; SurrealDB may validate permission expressions differently
    // depending on the field type.
    validate_query(&format!(
        "DEFINE FIELD __aurora__ ON __aurora__ TYPE string PERMISSIONS FOR {operation} {}",
        body
    ))
}

fn validate_query(query: &str) -> Result<(), SurqlParseError> {
    surrealdb_core::syn::parse(query)
        .map(|_| ())
        .map_err(|error| SurqlParseError {
            message: format_surql_query_error(error),
        })
}

fn format_surql_error(error: impl fmt::Display) -> String {
    explain_surql_error(error.to_string())
}

fn format_surql_query_error(error: surrealdb_core::err::Error) -> String {
    match error {
        surrealdb_core::err::Error::InvalidQuery(error) => {
            let message = error
                .errors
                .first()
                .cloned()
                .unwrap_or_else(|| "SurrealQL parse error".to_string());
            explain_surql_error(message)
        }
        error => format_surql_error(error),
    }
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
            "write a SurrealQL expression after this keyword, for example `WHERE $value != NONE` or `WHERE $auth.role = \"admin\"`",
        );
    }
    None
}
