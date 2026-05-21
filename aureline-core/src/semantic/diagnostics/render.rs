/// Rendering contract for semantic diagnostic domain enums.
///
/// Each diagnostic family should store structured data and implement this trait
/// to turn that data into user-facing text. Keeping this as a trait makes new
/// diagnostic families opt into both message and hint rendering explicitly.
pub(crate) trait RenderSemanticDiagnostic {
    fn message(&self) -> String;

    fn hint(&self) -> Option<String>;
}
