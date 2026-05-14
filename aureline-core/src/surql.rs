//! SurrealQL validation helpers.
//!
//! Aureline owns the surrounding schema syntax. SurrealDB owns the escaped
//! `#surql` body syntax, so validation delegates to SurrealDB's parser.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurqlParseError {
    pub message: String,
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
        body
    ))
}

fn validate_query(query: &str) -> Result<(), SurqlParseError> {
    surrealdb_core::syn::parse(query)
        .map(|_| ())
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
            "write a SurrealQL expression after this keyword, for example `WHERE $value != NONE` or `WHERE $auth.role = \"admin\"`",
        );
    }
    None
}
