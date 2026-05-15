use serde::{Deserialize, Serialize};

use crate::check::diagnostics::SourceRange;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    pub items: Vec<SchemaItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SchemaItem {
    #[serde(rename = "doc_comment")]
    DocComment { text: String },
    #[serde(rename = "table")]
    TableDecl(Table),
    #[serde(rename = "analyzer")]
    AnalyzerDecl(Analyzer),
    #[serde(rename = "function")]
    FunctionDecl(Function),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurqlBlock {
    pub body: String,
}

/// Top-level user-defined function declaration. Aureline owns the typed
/// signature; the body remains a raw SurQL escape hatch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    #[serde(skip)]
    pub source_range: Option<SourceRange>,
    #[serde(skip)]
    pub name_range: Option<SourceRange>,
    pub params: Vec<FunctionParam>,
    #[serde(rename = "return")]
    pub return_type: Type,
    pub body: SurqlBlock,
    /// Function-level `@@` attributes. Currently only `@@allow` is supported.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_attributes: Vec<Attribute>,
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.params == other.params
            && self.return_type == other.return_type
            && self.body == other.body
            && self.raw_attributes == other.raw_attributes
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParam {
    pub name: String,
    #[serde(skip)]
    pub name_range: Option<SourceRange>,
    #[serde(rename = "type")]
    pub ty: Type,
}

impl PartialEq for FunctionParam {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.ty == other.ty
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    #[serde(skip)]
    pub source_range: Option<SourceRange>,
    #[serde(skip)]
    pub name_range: Option<SourceRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<String>,
    pub fields: Vec<Field>,
    /// Indexes on this table. Populated by semantic lowering from `@`/`@@`
    /// attributes; empty in the raw post-parse AST.
    #[serde(default)]
    pub indexes: Vec<Index>,
    /// Block-level `@@`-attributes the user wrote. Kept around for the LSP
    /// (hover, completion, structured diagnostics) and for error messages
    /// that point back at the original source. Semantic lowering consumes these
    /// to populate `indexes`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_attributes: Vec<Attribute>,
}

impl PartialEq for Table {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.modifier == other.modifier
            && self.fields == other.fields
            && self.indexes == other.indexes
            && self.raw_attributes == other.raw_attributes
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(skip)]
    pub source_range: Option<SourceRange>,
    #[serde(skip)]
    pub name_range: Option<SourceRange>,
    #[serde(rename = "type")]
    pub ty: Type,
    /// True for trailing-`?` syntax (`int?`). Top-level `option<T>` is normalized
    /// to this flag during parsing — `option<int>` and `int?` produce identical
    /// AST. `Type::Option` only appears nested inside compound types.
    pub optional: bool,
    /// True if `@flexible` was applied. Only valid when `ty` is `object`;
    /// emitted as `TYPE object FLEXIBLE`. Validator enforces the constraint.
    #[serde(default, skip_serializing_if = "is_false")]
    pub flexible: bool,
    /// Field-level `@`-attributes the user wrote. See `Table::raw_attributes`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub raw_attributes: Vec<Attribute>,
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.ty == other.ty
            && self.optional == other.optional
            && self.flexible == other.flexible
            && self.raw_attributes == other.raw_attributes
    }
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Top-level analyzer declaration. Mirrors SurrealDB's
/// `DEFINE ANALYZER name TOKENIZERS ... FILTERS ...`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analyzer {
    pub name: String,
    #[serde(skip)]
    pub source_range: Option<SourceRange>,
    #[serde(skip)]
    pub name_range: Option<SourceRange>,
    #[serde(default)]
    pub tokenizers: Vec<String>,
    #[serde(default)]
    pub filters: Vec<FilterCall>,
}

impl PartialEq for Analyzer {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.tokenizers == other.tokenizers
            && self.filters == other.filters
    }
}

/// A filter applied to an analyzer. May or may not have args (e.g.
/// `lowercase` vs `snowball(english)` vs `edgengram(1, 3)`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterCall {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

/// Generic attribute as parsed from source. The grammar produces these
/// without knowing what they mean; semantic lowering interprets them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<AttributeArg>,
    #[serde(skip)]
    pub source_range: Option<SourceRange>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AttributeArg {
    Keyword { name: String, value: AttributeValue },
    Positional { value: AttributeValue },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AttributeValue {
    Number {
        value: f64,
    },
    Bool {
        value: bool,
    },
    /// Bare identifier, e.g. `cosine`, `f32`, `edu_analyzer`.
    Ident {
        value: String,
    },
    String {
        value: String,
    },
    Surql {
        body: String,
        #[serde(skip)]
        source_range: Option<SourceRange>,
    },
    Array {
        values: Vec<AttributeValue>,
    },
    /// Parens-wrapped value list — mirrors SurrealDB's `BM25(1.2, 0.75)`.
    Tuple {
        values: Vec<AttributeValue>,
    },
}

/// A SurrealDB index, post-validation. Built from the user's `@`/`@@`
/// attributes by semantic lowering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub fields: Vec<String>,
    pub kind: IndexKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IndexKind {
    Standard,
    Unique,
    Count,
    Fulltext {
        analyzer: String,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        bm25: Option<Bm25>,
        #[serde(default, skip_serializing_if = "is_false")]
        highlights: bool,
    },
    Hnsw {
        dimension: u32,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        dist: Option<String>,
        #[serde(rename = "type", skip_serializing_if = "Option::is_none", default)]
        ty: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        efc: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        m: Option<u32>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bm25 {
    pub k1: f64,
    pub b: f64,
}

/// Type expression, mirroring SurrealDB's data type system.
///
/// Serialization uses `tag = "kind"` so each variant is self-describing:
///   { "kind": "primitive", "name": "string" }
///   { "kind": "option", "inner": { "kind": "primitive", "name": "int" } }
///   { "kind": "array", "inner": ..., "length": 5 }
///   { "kind": "record", "table": "user" }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Type {
    /// Primitive types by name: bool, int, float, decimal, number, string,
    /// datetime, duration, uuid, bytes, any, regex, object, range.
    Primitive { name: String },
    /// `option<T>` — only appears nested. Top-level optional is the
    /// `Field::optional` flag.
    Option { inner: Box<Type> },
    /// `array<T>` or `array<T, N>`.
    Array {
        inner: Box<Type>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        length: Option<u64>,
    },
    /// `set<T>` or `set<T, N>`.
    Set {
        inner: Box<Type>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        length: Option<u64>,
    },
    /// `record` (any) or `record<table>` (constrained).
    Record {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        table: Option<String>,
    },
    /// `geometry<feature1 | feature2 | ...>`. Feature names are e.g. point,
    /// line, polygon, multipoint, multiline, multipolygon, feature, collection.
    Geometry { features: Vec<String> },
}

impl Type {
    /// Convenience constructor for a primitive type.
    pub fn primitive(name: impl Into<String>) -> Self {
        Type::Primitive { name: name.into() }
    }
}
