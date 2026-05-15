use crate::ast::{Attribute, AttributeArg, AttributeValue, Bm25, Field, Index, IndexKind};

use super::super::SemanticError;
use super::attributes::{
    at_attr, auto_name, err, err_at, ident_value, is_string, name_value, number_value, type_label,
};
use super::indexes::LoweredIndex;

pub(super) fn lower(
    table: &str,
    field: &Field,
    attr: &Attribute,
    indexes: &mut Vec<LoweredIndex>,
    errors: &mut Vec<SemanticError>,
) {
    match parse_fulltext_args(&attr.args).map_err(|error| at_attr(error, attr)) {
        Ok((kind, name_override)) => {
            if !is_string(&field.ty) {
                errors.push(err_at(
                    attr,
                    format!(
                        "@fulltext on {table}.{field} requires `string`, got `{ty}`",
                        field = field.name,
                        ty = type_label(&field.ty),
                    ),
                ));
                return;
            }
            indexes.push(LoweredIndex::new(
                Index {
                    name: name_override.unwrap_or_else(|| {
                        auto_name(table, std::slice::from_ref(&field.name), "fts")
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

// Mirrors SurrealDB's `SEARCH ANALYZER edu BM25(1.2, 0.75) HIGHLIGHTS`. The
// `bm25:` value is a tuple `(k1, b)` matching SurrealDB's `BM25(k1, b)`
// literal; omitting it means "use SurrealDB's defaults".
fn parse_fulltext_args(
    args: &[AttributeArg],
) -> Result<(IndexKind, Option<String>), SemanticError> {
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

fn parse_bm25_tuple(value: &AttributeValue) -> Result<Bm25, SemanticError> {
    let AttributeValue::Tuple { values } = value else {
        return Err(err(
            "`bm25` expects `(k1, b)` - two floats in parens".to_string()
        ));
    };
    if values.len() != 2 {
        return Err(err(format!(
            "`bm25` expects `(k1, b)` - exactly two floats, got {}",
            values.len()
        )));
    }
    let k1 = number_value(&values[0], "bm25 k1")?;
    let b = number_value(&values[1], "bm25 b")?;
    Ok(Bm25 { k1, b })
}
