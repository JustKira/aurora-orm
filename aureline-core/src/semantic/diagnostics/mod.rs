//! Shared semantic diagnostic constructors.
//!
//! Keep this module focused on semantic-domain diagnostics. The check layer is
//! still responsible for turning these into user-facing `Diagnostic` values.

mod analysis;
mod attributes;
mod render;
mod suggestions;

pub use analysis::AnalysisDiagnosticKind;
pub(crate) use analysis::{
    duplicate_analyzer_name, duplicate_field_name, duplicate_normalized_table_name,
    duplicate_table_name, function_body_param_mismatch, invalid_function_body_params,
    reserved_function_param, unknown_analyzer, unknown_record_table, unknown_surql_variable,
};
pub use attributes::{AttributeDiagnosticKind, AttributeScope};
pub(crate) use attributes::{invalid_attribute_usage, unknown_attribute};
pub(crate) use render::RenderSemanticDiagnostic;
