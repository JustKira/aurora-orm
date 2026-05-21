//! Shared semantic diagnostic constructors.
//!
//! Keep this module focused on semantic-domain diagnostics. The check layer is
//! still responsible for turning these into user-facing `Diagnostic` values.

mod attributes;
mod render;
mod suggestions;

pub use attributes::{AttributeDiagnosticKind, AttributeScope};
pub(crate) use attributes::{invalid_attribute_usage, unknown_attribute};
pub(crate) use render::RenderSemanticDiagnostic;
