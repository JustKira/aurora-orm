#![allow(dead_code)]

//! Shared helpers for the semantic test suite.
//!
//! `raw_ast` proves the parser/converter accepted the source without semantic
//! interpretation. `checked_schema` proves the full parse + semantic/lowering
//! pipeline accepted the source. `semantic_errors` asserts the parser passed,
//! but the schema should fail because its meaning is invalid.

use aureline_core::ast::{Schema, Table};
use aureline_core::{AurelineError, ValidationError};
use aureline_test_support::extract_table;

#[track_caller]
pub fn raw_ast(source: &str) -> Schema {
    aureline_core::parse_to_ast(source).expect("source should parse into raw AST")
}

#[track_caller]
pub fn checked_schema(source: &str) -> Schema {
    aureline_core::parse_validated(source).expect("source should pass semantic validation")
}

#[track_caller]
pub fn semantic_errors(source: &str) -> Vec<ValidationError> {
    let err = match aureline_core::parse_validated(source) {
        Ok(schema) => panic!("expected semantic validation error, got schema: {schema:#?}"),
        Err(error) => error,
    };
    let AurelineError::Validation(errors) = err else {
        panic!("expected semantic validation error, got {err:?}");
    };
    errors
}

#[track_caller]
pub fn assert_semantic_error_contains(source: &str, expected: &str) {
    let errors = semantic_errors(source);
    assert!(
        errors.iter().any(|error| error.message.contains(expected)),
        "expected semantic error containing `{expected}`, got {errors:#?}"
    );
}

#[track_caller]
pub fn assert_no_semantic_errors(source: &str) -> Schema {
    checked_schema(source)
}

#[track_caller]
pub fn table(schema: &Schema, name: &str) -> Table {
    extract_table(schema, name)
}
