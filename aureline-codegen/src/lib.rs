//! Aureline codegen placeholder.
//!
//! The in-house proof of concept starts with parser and language primitives.
//! Language clients will be designed after the schema IR is exercised internally.

use aureline_core::Schema;

pub fn describe_codegen_targets(_schema: &Schema) -> Vec<&'static str> {
    vec!["rust", "typescript", "python"]
}
