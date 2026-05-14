use crate::ast::Type;

pub fn pascal_to_snake(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_lower = false;

    for ch in s.chars() {
        if prev_lower && ch.is_ascii_uppercase() {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
        prev_lower = ch.is_ascii_lowercase();
    }

    out
}

/// Render a `Type` as the SurrealQL type expression that would appear after
/// `TYPE` in a `DEFINE FIELD` / `ALTER FIELD` statement.
pub fn surql_type(ty: &Type) -> String {
    match ty {
        Type::Primitive { name } => name.clone(),
        Type::Option { inner } => format!("option<{}>", surql_type(inner)),
        Type::Array {
            inner,
            length: None,
        } => format!("array<{}>", surql_type(inner)),
        Type::Array {
            inner,
            length: Some(n),
        } => format!("array<{}, {}>", surql_type(inner), n),
        Type::Set {
            inner,
            length: None,
        } => format!("set<{}>", surql_type(inner)),
        Type::Set {
            inner,
            length: Some(n),
        } => format!("set<{}, {}>", surql_type(inner), n),
        Type::Record { table: None } => "record".to_string(),
        Type::Record { table: Some(t) } => format!("record<{}>", pascal_to_snake(t)),
        Type::Geometry { features } => format!("geometry<{}>", features.join(" | ")),
    }
}
