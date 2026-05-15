#![allow(dead_code)]

//! Shared helpers for the semantic test suite.
//!
//! `raw_ast` proves the parser/converter accepted the source without semantic
//! interpretation. `checked_schema` proves the full parse + semantic/lowering
//! pipeline accepted the source. `semantic_errors` asserts the parser passed,
//! but the schema should fail because its meaning is invalid.

use aureline_core::ValidationError;
use aureline_core::ast::{Schema, Table};
use aureline_test_support::extract_table;

#[track_caller]
pub fn raw_ast(source: &str) -> Schema {
    aureline_core::parse_to_ast(source).expect("source should parse into raw AST")
}

#[track_caller]
pub fn checked_schema(source: &str) -> Schema {
    let mut schema = raw_ast(source);
    aureline_core::semantic::validate(&mut schema).expect("source should pass semantic validation");
    schema
}

#[track_caller]
pub fn semantic_errors(source: &str) -> Vec<ValidationError> {
    let mut schema = raw_ast(source);
    match aureline_core::semantic::validate(&mut schema) {
        Ok(()) => panic!("expected semantic validation error, got schema: {schema:#?}"),
        Err(errors) => errors,
    }
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
