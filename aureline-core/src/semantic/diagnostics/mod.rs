//! Shared semantic diagnostic constructors.
//!
//! Keep this module focused on semantic-domain diagnostics. The check layer is
//! still responsible for turning these into user-facing `Diagnostic` values.

mod attributes;
mod suggestions;

pub(crate) use attributes::unknown_attribute;
pub(crate) use suggestions::closest_match;
