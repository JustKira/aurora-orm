use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use aureline_core::ast::{Field, Schema, SchemaItem, Table, Type};
use aureline_core::{AurelineError, Diagnostic, DiagnosticCode, Severity, ValidationError};

#[macro_export]
macro_rules! expected_surql {
    ($($line:literal),* $(,)?) => {
        concat!($($line, "\n",)*)
    };
}

#[macro_export]
macro_rules! aureline_schema {
    ($($line:literal),* $(,)?) => {
        concat!($($line, "\n",)*)
    };
}

pub fn parse_schema(source: &str) -> Schema {
    aureline_core::parse_validated(source).unwrap()
}

pub fn validation_errors(source: &str) -> Vec<ValidationError> {
    let err = aureline_core::parse_validated(source).unwrap_err();
    let AurelineError::Validation(errors) = err else {
        panic!("expected validation error, got {err:?}");
    };
    errors
}

pub fn empty_schema() -> Schema {
    Schema { items: Vec::new() }
}

pub fn schema(tables: Vec<Table>) -> Schema {
    Schema {
        items: tables.into_iter().map(SchemaItem::TableDecl).collect(),
    }
}

pub fn table(name: &str, modifier: Option<&str>, fields: Vec<Field>) -> Table {
    Table {
        name: name.to_string(),
        source_range: None,
        name_range: None,
        modifier: modifier.map(str::to_string),
        fields,
        indexes: Vec::new(),
        raw_attributes: Vec::new(),
    }
}

pub fn field(name: &str, type_name: &str, optional: bool) -> Field {
    Field {
        name: name.to_string(),
        source_range: None,
        name_range: None,
        ty: Type::primitive(type_name),
        optional,
        flexible: false,
        always: false,
        default: None,
        raw_attributes: Vec::new(),
    }
}

pub fn extract_table(schema: &Schema, name: &str) -> Table {
    schema
        .items
        .iter()
        .find_map(|item| match item {
            SchemaItem::TableDecl(table) if table.name == name => Some(table.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("table {name} not in schema"))
}

pub fn diagnostics_for(source: &str) -> Vec<Diagnostic> {
    let report = aureline_core::check(source);
    eprintln!("source:\n{source}\n");
    for diagnostic in &report.diagnostics {
        eprintln!("diagnostic: {diagnostic:#?}");
    }
    report.diagnostics
}

pub struct ExpectedDiagnostic<'a> {
    pub code: DiagnosticCode,
    pub message: &'a str,
    pub start: (u32, u32),
    pub end: (u32, u32),
}

pub fn assert_single_diagnostic(diagnostics: &[Diagnostic], expected: ExpectedDiagnostic<'_>) {
    let diagnostic = only_diagnostic(diagnostics);
    assert_eq!(diagnostic.code, expected.code);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(diagnostic.message, expected.message);
    assert_range(diagnostic, expected.start, expected.end);
}

pub fn only_diagnostic(diagnostics: &[Diagnostic]) -> &Diagnostic {
    assert_eq!(
        diagnostics.len(),
        1,
        "expected one diagnostic, got {diagnostics:#?}"
    );
    &diagnostics[0]
}

pub fn assert_range(diagnostic: &Diagnostic, start: (u32, u32), end: (u32, u32)) {
    assert_eq!(diagnostic.range.start.line, start.0);
    assert_eq!(diagnostic.range.start.character, start.1);
    assert_eq!(diagnostic.range.end.line, end.0);
    assert_eq!(diagnostic.range.end.character, end.1);
}

pub fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("aureline_{label}_{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}
