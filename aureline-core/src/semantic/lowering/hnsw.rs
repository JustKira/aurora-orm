use crate::ast::{Attribute, AttributeArg, AttributeValue, Field, Index, IndexKind};

use super::super::SemanticError;
use super::attributes::{
    at_attr, auto_name, err, err_at, ident_value, is_array_of_float, name_value, type_label,
    uint_value_bounded,
};
use super::indexes::LoweredIndex;

pub(super) fn lower(
    table: &str,
    field: &Field,
    attr: &Attribute,
    indexes: &mut Vec<LoweredIndex>,
    errors: &mut Vec<SemanticError>,
) {
    match parse_hnsw_args(&attr.args).map_err(|error| at_attr(error, attr)) {
        Ok((kind, name_override)) => {
            if !is_array_of_float(&field.ty) {
                errors.push(err_at(
                    attr,
                    format!(
                        "@hnsw on {table}.{field} requires `array<float>`, got `{ty}`",
                        field = field.name,
                        ty = type_label(&field.ty),
                    ),
                ));
                return;
            }
            indexes.push(LoweredIndex::new(
                Index {
                    name: name_override.unwrap_or_else(|| {
                        auto_name(table, std::slice::from_ref(&field.name), "hnsw")
                    }),
                    fields: vec![field.name.clone()],
                    kind,
                },
                attr,
            ));
        }
        Err(error) => errors.push(error),
    }
}

// Mirrors SurrealDB's `HNSW DIMENSION 1536 DIST cosine TYPE F32 EFC 200 M 16`.
// Returns the kind + the optional explicit `name:` so the caller can decide
// auto-naming.
fn parse_hnsw_args(args: &[AttributeArg]) -> Result<(IndexKind, Option<String>), SemanticError> {
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
            "dimension" => {
                dimension = Some(uint_value_bounded(
                    value,
                    "@hnsw dimension",
                    u16::MAX as u32,
                )?)
            }
            "dist" => dist = Some(hnsw_distance_value(value)?),
            "type" => ty = Some(hnsw_vector_type_value(value)?),
            "efc" => efc = Some(uint_value_bounded(value, "@hnsw efc", u16::MAX as u32)?),
            "m" => m = Some(uint_value_bounded(value, "@hnsw m", 127)?),
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

fn hnsw_distance_value(value: &AttributeValue) -> Result<String, SemanticError> {
    const DISTANCES: &[&str] = &[
        "chebyshev",
        "cosine",
        "euclidean",
        "hamming",
        "jaccard",
        "manhattan",
        "pearson",
    ];

    let value = ident_value(value, "@hnsw dist")?;
    let normalized = value.to_ascii_lowercase();
    if DISTANCES.contains(&normalized.as_str()) {
        return Ok(normalized);
    }
    if normalized == "minkowski" {
        return Err(err(
            "unsupported @hnsw dist `minkowski`; SurrealDB requires `MINKOWSKI <number>`, which Aureline does not model yet"
                .to_string(),
        ));
    }
    Err(err(format!(
        "unknown @hnsw dist `{value}`; expected one of: {}",
        DISTANCES.join(", ")
    )))
}

fn hnsw_vector_type_value(value: &AttributeValue) -> Result<String, SemanticError> {
    const TYPES: &[&str] = &["f64", "f32", "i64", "i32", "i16"];

    let value = ident_value(value, "@hnsw type")?;
    let normalized = value.to_ascii_lowercase();
    if TYPES.contains(&normalized.as_str()) {
        return Ok(normalized);
    }
    Err(err(format!(
        "unknown @hnsw type `{value}`; expected one of: {}",
        TYPES.join(", ")
    )))
}
