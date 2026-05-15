//! SurrealQL validation helpers.
//!
//! Aureline owns the surrounding schema syntax. SurrealDB owns the escaped
//! `#surql` body syntax, so validation delegates to SurrealDB's parser.

use std::collections::BTreeSet;
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

pub fn validate_function_permission(body: &str) -> Result<(), SurqlParseError> {
    validate_query(&format!(
        "DEFINE FUNCTION fn::__aureline__() {{ RETURN NONE; }} PERMISSIONS {}",
        body.trim()
    ))
}

pub fn function_body_params(body: &str) -> Result<BTreeSet<String>, SurqlParseError> {
    surrealdb_core::syn::parse(body.trim()).map_err(|error| SurqlParseError {
        message: format_surql_error(error),
    })?;
    Ok(collect_params(body))
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

pub(crate) fn is_builtin_param(name: &str) -> bool {
    matches!(
        name,
        "after"
            | "auth"
            | "before"
            | "event"
            | "input"
            | "parent"
            | "session"
            | "this"
            | "token"
            | "value"
    )
}

fn collect_params(body: &str) -> BTreeSet<String> {
    let mut scanner = ParamScanner::new(body);
    scanner.scan()
}

struct ParamScanner<'a> {
    input: &'a str,
    pos: usize,
    params: BTreeSet<String>,
}

impl<'a> ParamScanner<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            params: BTreeSet::new(),
        }
    }

    fn scan(&mut self) -> BTreeSet<String> {
        while let Some(ch) = self.peek_char() {
            match ch {
                '\'' | '"' | '`' => {
                    self.advance_char();
                    self.skip_quoted(ch);
                }
                '/' if self.peek_next_char() == Some('/') => self.skip_line_comment(),
                '/' if self.peek_next_char() == Some('*') => self.skip_block_comment(),
                '$' => {
                    self.advance_char();
                    self.read_param();
                }
                _ => {
                    self.advance_char();
                }
            }
        }
        std::mem::take(&mut self.params)
    }

    fn read_param(&mut self) {
        let Some(first) = self.peek_char() else {
            return;
        };
        if !is_identifier_start(first) {
            return;
        }

        let start = self.pos;
        self.advance_char();
        while let Some(ch) = self.peek_char() {
            if !is_identifier_continue(ch) {
                break;
            }
            self.advance_char();
        }

        let name = &self.input[start..self.pos];
        if !is_builtin_param(name) {
            self.params.insert(name.to_string());
        }
    }

    fn skip_quoted(&mut self, quote: char) {
        let mut escaped = false;
        while let Some(ch) = self.advance_char() {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == quote {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        self.advance_char();
        self.advance_char();
        while let Some(ch) = self.advance_char() {
            if ch == '\n' {
                break;
            }
        }
    }

    fn skip_block_comment(&mut self) {
        self.advance_char();
        self.advance_char();
        let mut previous = '\0';
        while let Some(ch) = self.advance_char() {
            if previous == '*' && ch == '/' {
                break;
            }
            previous = ch;
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn peek_next_char(&self) -> Option<char> {
        let mut chars = self.input[self.pos..].chars();
        chars.next()?;
        chars.next()
    }

    fn advance_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
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
