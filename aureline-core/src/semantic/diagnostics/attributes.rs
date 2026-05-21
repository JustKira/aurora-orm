use crate::semantic::{AttributeScope, SemanticDiagnostic, SemanticDiagnosticKind, SemanticError};

pub(crate) fn unknown_attribute(
    scope: AttributeScope,
    name: &str,
    valid: &'static [&'static str],
) -> SemanticError {
    SemanticDiagnostic::new(SemanticDiagnosticKind::UnknownAttribute {
        scope,
        name: name.to_string(),
        valid,
    })
}
