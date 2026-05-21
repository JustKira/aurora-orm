use aureline_test_support::validation_errors;

// SurrealDB DEFINE INDEX also has statement-level clauses like OVERWRITE,
// IF NOT EXISTS, COMMENT, CONCURRENTLY, DEFER, and the COLUMNS alias. Aureline
// does not model those in schema attributes yet, so these modules focus on the
// index definitions Aureline can currently express and emit.

#[path = "definitions/compound_index.rs"]
mod compound_index;
#[path = "definitions/fulltext_index.rs"]
mod fulltext_index;
#[path = "definitions/table_field_index.rs"]
mod table_field_index;
#[path = "definitions/vector_index.rs"]
mod vector_index;

fn assert_validation_contains(source: &str, expected: &str) {
    let errors = validation_errors(source);
    assert!(
        errors
            .iter()
            .any(|error| error.message().contains(expected)),
        "expected validation error containing `{expected}`, got {errors:#?}"
    );
}
