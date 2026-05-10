//! Aurora core: parser, validator, and serializable AST for the Aurora language.

pub mod ast;
pub mod check;
mod convert;
pub mod emit;
pub mod error;
pub mod grammar;
pub mod validate;

pub use ast::*;
pub use check::CheckReport;
pub use check::check;
pub use check::diagnostics::{
    Diagnostic, DiagnosticCode, ParseDiagnostic, Severity, SourcePosition, SourceRange,
};
pub use error::AuroraError;
pub use grammar::Rule;
pub use validate::ValidationError;

use from_pest::FromPest;
use pest::Parser;

pub fn parse(source: &str) -> Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> {
    grammar::AuroraParser::parse(Rule::schema, source)
}

/// Parse without running the validator. The returned `Schema` has raw
/// `@`/`@@` attribute blobs on fields and tables; `Table.indexes` is empty
/// and `Field.flexible` is false until validation runs. Useful for the LSP
/// (which wants structure even from incomplete input).
pub fn parse_to_ast(source: &str) -> Result<Schema, AuroraError> {
    let mut pairs = parse(source)
        .map_err(check::syntax::parse_diagnostic_from_pest)
        .map_err(AuroraError::Parse)?;
    let parsed = convert::Schema::from_pest(&mut pairs)
        .map_err(|error| AuroraError::Convert(format!("{error:?}")))?;

    Ok(parsed.into_ast())
}

/// Parse + validate. Lowers raw attributes into structured `Index`/`flexible`
/// fields. This is what `aurora-migrate` and the CLI consume.
pub fn parse_validated(source: &str) -> Result<Schema, AuroraError> {
    let mut schema = parse_to_ast(source)?;
    validate::validate(&mut schema).map_err(AuroraError::Validation)?;
    Ok(schema)
}

pub fn parse_to_json(source: &str) -> Result<String, AuroraError> {
    let schema = parse_validated(source)?;
    serde_json::to_string_pretty(&schema).map_err(AuroraError::Json)
}

pub fn parse_schema(source: &str) -> Result<Schema, AuroraError> {
    parse_validated(source)
}
