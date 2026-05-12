//! Attribute rule book — turns raw `@`/`@@` attributes from the parser into
//! structured `Index`/`flexible` fields on the AST.
//!
//! This is the central place new attributes get added. Grammar stays
//! unchanged; new rules add a case to `validate_field_attribute` or
//! `validate_block_attribute`.

use std::fmt;

use crate::ast::{
    Attribute, AttributeArg, AttributeValue, Bm25, Field, Index, IndexKind, Schema, SchemaItem,
    Table, Type,
};
use crate::check::diagnostics::SourceRange;

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub message: String,
    /// Human-readable hint, e.g. "did you mean `@hnsw`?"
    pub hint: Option<String>,
    pub range: Option<SourceRange>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(hint) = &self.hint {
            write!(f, " — {hint}")?;
        }
        Ok(())
    }
}

/// Validate a parsed schema: lower raw `@`/`@@` attributes into structured
/// `indexes` (on tables) and `flexible` flags (on fields). Returns all
/// errors at once so users see every problem in one shot.
pub fn validate(schema: &mut Schema) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    for item in &mut schema.items {
        if let SchemaItem::TableDecl(table) = item {
            validate_table(table, &mut errors);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_table(table: &mut Table, errors: &mut Vec<ValidationError>) {
    let table_name = table.name.clone();
    let mut indexes = Vec::new();

    // Field-level annotations.
    for field in &mut table.fields {
        validate_field_attributes(&table_name, field, &mut indexes, errors);
    }

    // Block-level annotations.
    for attr in &table.raw_attributes {
        validate_block_attribute(&table_name, &table.fields, attr, &mut indexes, errors);
    }

    table.indexes = indexes;
}

const FIELD_ATTRS: &[&str] = &[
    "unique", "index", "flexible", "hnsw", "fulltext", "assert", "allow",
];

fn validate_field_attributes(
    table: &str,
    field: &mut Field,
    indexes: &mut Vec<Index>,
    errors: &mut Vec<ValidationError>,
) {
    for attr in &field.raw_attributes {
        match attr.name.as_str() {
            "unique" => match parse_named_only(&attr.args, "@unique").map_err(|e| e.at(attr)) {
                Ok(name_override) => indexes.push(Index {
                    name: name_override
                        .unwrap_or_else(|| auto_name(table, &[field.name.clone()], "unique")),
                    fields: vec![field.name.clone()],
                    kind: IndexKind::Unique,
                }),
                Err(e) => errors.push(e),
            },
            "index" => match parse_named_only(&attr.args, "@index").map_err(|e| e.at(attr)) {
                Ok(name_override) => indexes.push(Index {
                    name: name_override
                        .unwrap_or_else(|| auto_name(table, &[field.name.clone()], "idx")),
                    fields: vec![field.name.clone()],
                    kind: IndexKind::Standard,
                }),
                Err(e) => errors.push(e),
            },
            "flexible" => {
                if !attr.args.is_empty() {
                    errors.push(err_at(
                        attr,
                        format!(
                            "@flexible on {table}.{f} takes no arguments",
                            f = field.name
                        ),
                    ));
                    continue;
                }
                if !is_object(&field.ty) {
                    errors.push(err_at(
                        attr,
                        format!(
                            "@flexible on {table}.{f} requires `object`, got `{ty}`",
                            f = field.name,
                            ty = type_label(&field.ty),
                        ),
                    ));
                    continue;
                }
                field.flexible = true;
            }
            "hnsw" => match parse_hnsw_args(&attr.args).map_err(|e| e.at(attr)) {
                Ok((kind, name_override)) => {
                    if !is_array_of_float(&field.ty) {
                        errors.push(err_at(
                            attr,
                            format!(
                                "@hnsw on {table}.{f} requires `array<float>`, got `{ty}`",
                                f = field.name,
                                ty = type_label(&field.ty),
                            ),
                        ));
                        continue;
                    }
                    indexes.push(Index {
                        name: name_override
                            .unwrap_or_else(|| auto_name(table, &[field.name.clone()], "hnsw")),
                        fields: vec![field.name.clone()],
                        kind,
                    });
                }
                Err(e) => errors.push(e),
            },
            "fulltext" => match parse_fulltext_args(&attr.args).map_err(|e| e.at(attr)) {
                Ok((kind, name_override)) => {
                    if !is_string(&field.ty) {
                        errors.push(err_at(
                            attr,
                            format!(
                                "@fulltext on {table}.{f} requires `string`, got `{ty}`",
                                f = field.name,
                                ty = type_label(&field.ty),
                            ),
                        ));
                        continue;
                    }
                    indexes.push(Index {
                        name: name_override
                            .unwrap_or_else(|| auto_name(table, &[field.name.clone()], "fts")),
                        fields: vec![field.name.clone()],
                        kind,
                    });
                }
                Err(e) => errors.push(e),
            },
            "assert" => match attr.args.as_slice() {
                [
                    AttributeArg::Positional {
                        value: AttributeValue::Surql { body, source_range },
                    },
                ] => {
                    if let Err(error) = crate::surql::validate_expression(body) {
                        errors.push(ValidationError {
                            message: error.message,
                            hint: None,
                            range: source_range.or(attr.source_range),
                        });
                    }
                }
                _ => {
                    errors.push(err_at(
                        attr,
                        "@assert expects exactly one `#surql { ... }` block".to_string(),
                    ));
                }
            },
            "allow" => {
                if let Err(error) = validate_allow_args(attr) {
                    errors.push(error);
                }
            }
            unknown => errors.push(unknown_attribute(unknown, FIELD_ATTRS, "field").at(attr)),
        }
    }
}

fn validate_allow_args(attr: &Attribute) -> Result<(), ValidationError> {
    let mut operation = None;
    let mut permission = None;

    for arg in &attr.args {
        match arg {
            AttributeArg::Keyword { name, value } if name == "op" => {
                if operation.is_some() {
                    return Err(err_at(
                        attr,
                        "@allow has duplicate `op:` arguments".to_string(),
                    ));
                }
                let AttributeValue::String { value } = value else {
                    return Err(err_at(
                        attr,
                        "@allow `op:` must be a string literal like \"SELECT\"".to_string(),
                    ));
                };
                operation = Some(value.as_str());
            }
            AttributeArg::Keyword { name, .. } => {
                return Err(err_at(
                    attr,
                    format!("unknown @allow arg `{name}`; expected `op`"),
                ));
            }
            AttributeArg::Positional {
                value: AttributeValue::Surql { body, source_range },
            } => {
                if permission.is_some() {
                    return Err(err_at(
                        attr,
                        "@allow has duplicate `#surql { ... }` permission blocks".to_string(),
                    ));
                }
                permission = Some((body, source_range));
            }
            AttributeArg::Positional { .. } => {
                return Err(err_at(
                    attr,
                    "@allow positional arguments must be `#surql { ... }`; use `op: \"SELECT\"` for the operation".to_string(),
                ));
            }
        }
    }

    let Some(operation) = operation else {
        return Err(err_at(
            attr,
            "@allow requires an `op: \"SELECT\"` argument".to_string(),
        ));
    };

    let Some((body, source_range)) = permission else {
        return Err(err_at(
            attr,
            "@allow requires one positional `#surql { ... }` permission block".to_string(),
        ));
    };

    let operation_keyword = operation.to_ascii_uppercase();
    match operation_keyword.as_str() {
        "SELECT" | "CREATE" | "UPDATE" | "DELETE" => {}
        _ => {
            return Err(err_at(
                attr,
                format!(
                    "unknown @allow operation `{operation}`; expected one of: SELECT, CREATE, UPDATE, DELETE"
                ),
            ));
        }
    }

    crate::surql::validate_field_permission(&operation_keyword, body).map_err(|error| {
        ValidationError {
            message: error.message,
            hint: None,
            range: source_range.or(attr.source_range),
        }
    })
}

/// `@unique` / `@index` accept only one optional kw arg: `name: "..."`.
fn parse_named_only(args: &[AttributeArg], label: &str) -> Result<Option<String>, ValidationError> {
    let mut name = None;
    for arg in args {
        match arg {
            AttributeArg::Keyword { name: key, value } => match key.as_str() {
                "name" => name = Some(name_value(value, &format!("{label} name"))?),
                other => {
                    return Err(err(format!(
                        "unknown {label} arg `{other}`; expected `name`"
                    )));
                }
            },
            AttributeArg::Positional { .. } => {
                return Err(err(format!("{label} does not accept positional arguments")));
            }
        }
    }
    Ok(name)
}

// Block-level annotations are only for *composite* or *table-level* concepts.
// Single-field indexes (fulltext, hnsw, plain index, unique) live as field-level
// `@`-annotations only. We deliberately don't accept `@@fulltext` / `@@hnsw`
// because they'd duplicate the field-level forms and invite drift.
const BLOCK_ATTRS: &[&str] = &["index", "unique", "count"];

fn validate_block_attribute(
    table: &str,
    fields: &[Field],
    attr: &Attribute,
    indexes: &mut Vec<Index>,
    errors: &mut Vec<ValidationError>,
) {
    match attr.name.as_str() {
        "count" => {
            if !attr.args.is_empty() {
                errors.push(err_at(
                    attr,
                    format!("@@count on {table} takes no arguments"),
                ));
                return;
            }
            indexes.push(Index {
                name: auto_name(table, &[], "count"),
                fields: Vec::new(),
                kind: IndexKind::Count,
            });
        }
        "index" => match parse_field_list_block(table, fields, &attr.args, "@@index")
            .map_err(|e| e.at(attr))
        {
            Ok((field_names, name_override)) => {
                indexes.push(Index {
                    name: name_override.unwrap_or_else(|| auto_name(table, &field_names, "idx")),
                    fields: field_names,
                    kind: IndexKind::Standard,
                });
            }
            Err(e) => errors.push(e),
        },
        "unique" => match parse_field_list_block(table, fields, &attr.args, "@@unique")
            .map_err(|e| e.at(attr))
        {
            Ok((field_names, name_override)) => {
                indexes.push(Index {
                    name: name_override.unwrap_or_else(|| auto_name(table, &field_names, "unique")),
                    fields: field_names,
                    kind: IndexKind::Unique,
                });
            }
            Err(e) => errors.push(e),
        },
        unknown => errors.push(unknown_attribute(unknown, BLOCK_ATTRS, "block").at(attr)),
    }
}

// === HNSW arg parsing (field-level, keyword-only) ===
//
// Mirrors SurrealDB's `HNSW DIMENSION 1536 DIST cosine TYPE F32 EFC 200 M 16`.
// Returns the kind + the optional explicit `name:` so the caller can decide
// auto-naming.

fn parse_hnsw_args(args: &[AttributeArg]) -> Result<(IndexKind, Option<String>), ValidationError> {
    let mut dimension: Option<u32> = None;
    let mut dist: Option<String> = None;
    let mut ty: Option<String> = None;
    let mut efc: Option<u32> = None;
    let mut m: Option<u32> = None;
    let mut name: Option<String> = None;

    for arg in args {
        let AttributeArg::Keyword { name: key, value } = arg else {
            return Err(err("@hnsw does not accept positional arguments".to_string()));
        };
        match key.as_str() {
            "dimension" => dimension = Some(uint_value(value, "@hnsw dimension")?),
            "dist" => dist = Some(ident_value(value, "@hnsw dist")?),
            "type" => ty = Some(ident_value(value, "@hnsw type")?),
            "efc" => efc = Some(uint_value(value, "@hnsw efc")?),
            "m" => m = Some(uint_value(value, "@hnsw m")?),
            "name" => name = Some(name_value(value, "@hnsw name")?),
            other => {
                return Err(err(format!(
                    "unknown @hnsw arg `{other}`; expected one of: dimension, dist, type, efc, m, name"
                )));
            }
        }
    }

    let dimension =
        dimension.ok_or_else(|| err("@hnsw requires a `dimension:` argument".to_string()))?;
    Ok((
        IndexKind::Hnsw {
            dimension,
            dist,
            ty,
            efc,
            m,
        },
        name,
    ))
}

// === Fulltext arg parsing (field-level, keyword-only) ===
//
// Mirrors SurrealDB's `SEARCH ANALYZER edu BM25(1.2, 0.75) HIGHLIGHTS`. The
// `bm25:` value is a tuple `(k1, b)` matching SurrealDB's `BM25(k1, b)`
// literal; omitting it means "use SurrealDB's defaults".

fn parse_fulltext_args(
    args: &[AttributeArg],
) -> Result<(IndexKind, Option<String>), ValidationError> {
    let mut analyzer: Option<String> = None;
    let mut bm25: Option<Bm25> = None;
    let mut highlights = false;
    let mut name: Option<String> = None;

    for arg in args {
        let AttributeArg::Keyword { name: key, value } = arg else {
            return Err(err(
                "@fulltext does not accept positional arguments".to_string()
            ));
        };
        match key.as_str() {
            "analyzer" => analyzer = Some(ident_value(value, "@fulltext analyzer")?),
            "bm25" => bm25 = Some(parse_bm25_tuple(value)?),
            "highlights" => match value {
                AttributeValue::Bool { value } => highlights = *value,
                _ => return Err(err("@fulltext highlights: expected bool".to_string())),
            },
            "name" => name = Some(name_value(value, "@fulltext name")?),
            other => {
                return Err(err(format!(
                    "unknown @fulltext arg `{other}`; expected one of: analyzer, bm25, highlights, name"
                )));
            }
        }
    }

    let analyzer =
        analyzer.ok_or_else(|| err("@fulltext requires an `analyzer:` argument".to_string()))?;
    Ok((
        IndexKind::Fulltext {
            analyzer,
            bm25,
            highlights,
        },
        name,
    ))
}

fn parse_bm25_tuple(value: &AttributeValue) -> Result<Bm25, ValidationError> {
    let AttributeValue::Tuple { values } = value else {
        return Err(err(
            "`bm25` expects `(k1, b)` — two floats in parens".to_string()
        ));
    };
    if values.len() != 2 {
        return Err(err(format!(
            "`bm25` expects `(k1, b)` — exactly two floats, got {}",
            values.len()
        )));
    }
    let k1 = number_value(&values[0], "bm25 k1")?;
    let b = number_value(&values[1], "bm25 b")?;
    Ok(Bm25 { k1, b })
}

// === Block-level helpers ===

fn parse_field_list_block(
    table: &str,
    fields: &[Field],
    args: &[AttributeArg],
    label: &str,
) -> Result<(Vec<String>, Option<String>), ValidationError> {
    let mut field_names: Option<Vec<String>> = None;
    let mut name: Option<String> = None;
    for arg in args {
        let AttributeArg::Keyword { name: key, value } = arg else {
            return Err(err(format!("{label} does not accept positional arguments")));
        };
        match key.as_str() {
            "fields" => {
                let AttributeValue::Array { values } = value else {
                    return Err(err(format!(
                        "{label} on {table}: `fields:` expects an array of identifiers"
                    )));
                };
                let mut names = Vec::with_capacity(values.len());
                for v in values {
                    match v {
                        AttributeValue::Ident { value } => names.push(value.clone()),
                        _ => {
                            return Err(err(format!(
                                "{label} on {table}: `fields:` array must contain identifiers"
                            )));
                        }
                    }
                }
                field_names = Some(names);
            }
            "name" => name = Some(name_value(value, &format!("{label} name"))?),
            other => {
                return Err(err(format!(
                    "unknown {label} arg `{other}`; expected one of: fields, name"
                )));
            }
        }
    }
    let field_names = field_names.ok_or_else(|| {
        err(format!(
            "{label} on {table} requires a `fields: [...]` argument"
        ))
    })?;
    for fname in &field_names {
        if !fields.iter().any(|f| &f.name == fname) {
            return Err(err(format!("{label} on {table}: unknown field `{fname}`")));
        }
    }
    Ok((field_names, name))
}

// === Value extractors ===

fn ident_value(v: &AttributeValue, label: &str) -> Result<String, ValidationError> {
    match v {
        AttributeValue::Ident { value } => Ok(value.clone()),
        _ => Err(err(format!("{label}: expected an identifier"))),
    }
}

/// Accept either an identifier or a string literal — used for places like
/// `name:` where users might write `name: foo` or `name: "foo"`.
fn name_value(v: &AttributeValue, label: &str) -> Result<String, ValidationError> {
    match v {
        AttributeValue::Ident { value } | AttributeValue::String { value } => Ok(value.clone()),
        _ => Err(err(format!("{label}: expected an identifier or string"))),
    }
}

fn uint_value(v: &AttributeValue, label: &str) -> Result<u32, ValidationError> {
    match v {
        AttributeValue::Number { value } => {
            if value.fract() != 0.0 || *value < 0.0 {
                return Err(err(format!(
                    "{label}: expected a non-negative integer, got {value}"
                )));
            }
            Ok(*value as u32)
        }
        _ => Err(err(format!("{label}: expected a non-negative integer"))),
    }
}

fn number_value(v: &AttributeValue, label: &str) -> Result<f64, ValidationError> {
    match v {
        AttributeValue::Number { value } => Ok(*value),
        _ => Err(err(format!("{label}: expected a number"))),
    }
}

// === Type predicates ===

fn is_object(t: &Type) -> bool {
    matches!(t, Type::Primitive { name } if name == "object")
}

fn is_string(t: &Type) -> bool {
    matches!(t, Type::Primitive { name } if name == "string")
}

fn is_array_of_float(t: &Type) -> bool {
    matches!(t, Type::Array { inner, .. } if matches!(inner.as_ref(), Type::Primitive { name } if name == "float"))
}

fn type_label(t: &Type) -> String {
    crate::emit::surql_type(t)
}

// === Helpers ===

fn err(message: String) -> ValidationError {
    ValidationError {
        message,
        hint: None,
        range: None,
    }
}

fn err_at(attr: &Attribute, message: String) -> ValidationError {
    err(message).at(attr)
}

fn unknown_attribute(name: &str, valid: &[&str], scope: &str) -> ValidationError {
    let suggestion = closest_match(name, valid);
    let prefix = if scope == "block" { "@@" } else { "@" };
    ValidationError {
        message: format!("unknown {scope} attribute `{prefix}{name}`"),
        hint: suggestion.map(|s| format!("did you mean `{prefix}{s}`?")),
        range: None,
    }
}

impl ValidationError {
    fn at(mut self, attr: &Attribute) -> Self {
        self.range = attr.source_range;
        self
    }
}

/// Cheap edit-distance for "did you mean" suggestions. Levenshtein with a
/// distance cap of 3 — anything further isn't really a typo.
fn closest_match(target: &str, candidates: &[&str]) -> Option<String> {
    let mut best: Option<(&str, usize)> = None;
    for c in candidates {
        let d = levenshtein(target, c);
        if d <= 3 && best.map_or(true, |(_, bd)| d < bd) {
            best = Some((c, d));
        }
    }
    best.map(|(s, _)| s.to_string())
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for (i, ac) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, bc) in b.iter().enumerate() {
            let cost = if ac == bc { 0 } else { 1 };
            curr[j + 1] = (curr[j] + 1).min(prev[j + 1] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

fn auto_name(table: &str, fields: &[String], suffix: &str) -> String {
    let table_part = crate::emit::pascal_to_snake(table);
    if fields.is_empty() {
        format!("{table_part}_{suffix}")
    } else {
        format!("{table_part}_{}_{suffix}", fields.join("_"))
    }
}
