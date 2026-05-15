//! SurrealQL validation helpers.
//!
//! Aureline owns the surrounding schema syntax. SurrealDB owns the escaped
//! `#surql` body syntax, so validation delegates to SurrealDB's parser.

use std::collections::BTreeSet;
use std::fmt;

use surrealdb_core::sql::visit::{Visit, Visitor};
use surrealdb_core::sql::Param;

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
    // TODO: infer this TYPE from the Aureline field type instead of hardcoding
    // `string`; SurrealDB may validate permission expressions differently
    // depending on the field type.
    validate_query(&format!(
        "DEFINE FIELD __aureline__ ON __aureline__ TYPE string PERMISSIONS FOR {operation} {}",
        body
    ))
}

pub fn function_body_params(body: &str) -> Result<BTreeSet<String>, SurqlParseError> {
    let query = surrealdb_core::syn::parse(body.trim()).map_err(|error| SurqlParseError {
        message: format_surql_query_error(error),
    })?;
    let mut visitor = ParamCollector::default();
    query.visit(&mut visitor).map_err(|error| SurqlParseError {
        message: error.to_string(),
    })?;
    Ok(visitor.params)
}

fn validate_query(query: &str) -> Result<(), SurqlParseError> {
    surrealdb_core::syn::parse(query)
        .map(|_| ())
        .map_err(|error| SurqlParseError {
            message: format_surql_query_error(error),
        })
}

#[derive(Default)]
struct ParamCollector {
    params: BTreeSet<String>,
}

impl Visitor for ParamCollector {
    type Error = fmt::Error;

    fn visit_param(&mut self, param: &Param) -> Result<(), Self::Error> {
        let name = param.as_str();
        if !is_builtin_param(name) {
            self.params.insert(name.to_string());
        }
        Ok(())
    }
}

fn is_builtin_param(name: &str) -> bool {
    matches!(
        name,
        "after" | "auth" | "before" | "event" | "input" | "parent" | "session" | "this" | "token"
            | "value"
    )
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
