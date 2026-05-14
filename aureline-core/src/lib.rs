//! Aureline core: parser, semantic validation, and serializable AST for the Aureline language.

pub mod ast;
pub mod check;
mod convert;
pub mod emit;
pub mod error;
pub mod grammar;
pub mod semantic;
pub mod surql;

pub use ast::*;
pub use check::CheckReport;
pub use check::check;
pub use check::diagnostics::{
    Diagnostic, DiagnosticCode, ParseDiagnostic, Severity, SourcePosition, SourceRange,
};
pub use error::AurelineError;
pub use grammar::Rule;
pub use semantic::SemanticError;
pub use semantic::SemanticError as ValidationError;

use from_pest::FromPest;
use pest::Parser;

pub fn parse(source: &str) -> Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> {
    grammar::AurelineParser::parse(Rule::schema, source)
}

pub(crate) fn parse_source_file(
    source: &str,
) -> Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> {
    grammar::AurelineParser::parse(Rule::source_file, source)
}

/// Parse without running semantic validation. The returned `Schema` has raw
/// `@`/`@@` attribute blobs on fields and tables; `Table.indexes` is empty
/// and `Field.flexible` is false until validation runs. Useful for the LSP
/// (which wants structure even from incomplete input).
pub fn parse_to_ast(source: &str) -> Result<Schema, AurelineError> {
    let pairs = parse(source)
        .map_err(check::syntax::parse_diagnostic_from_pest)
        .map_err(AurelineError::Parse)?;
    parse_pairs_to_ast(pairs)
}

pub(crate) fn parse_pairs_to_ast(
    mut pairs: pest::iterators::Pairs<'_, Rule>,
) -> Result<Schema, AurelineError> {
    let parsed = convert::Schema::from_pest(&mut pairs)
        .map_err(|error| AurelineError::Convert(format!("{error:?}")))?;

    Ok(parsed.into_ast())
}

pub(crate) fn parse_source_file_pairs_to_ast(
    mut pairs: pest::iterators::Pairs<'_, Rule>,
) -> Result<Schema, AurelineError> {
    let parsed = convert::SourceFile::from_pest(&mut pairs)
        .map_err(|error| AurelineError::Convert(format!("{error:?}")))?;

    Ok(parsed.into_ast())
}

/// Parse + validate. Lowers raw attributes into structured `Index`/`flexible`
/// fields. This is what `aureline-migrate` and the CLI consume.
pub fn parse_validated(source: &str) -> Result<Schema, AurelineError> {
    let mut schema = parse_to_ast(source)?;
    semantic::validate(&mut schema).map_err(AurelineError::Validation)?;
    Ok(schema)
}

pub fn parse_to_json(source: &str) -> Result<String, AurelineError> {
    let schema = parse_validated(source)?;
    serde_json::to_string_pretty(&schema).map_err(AurelineError::Json)
}

pub fn parse_schema(source: &str) -> Result<Schema, AurelineError> {
    parse_validated(source)
}
