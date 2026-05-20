use crate::ast::{
    Analyzer, Attribute, AttributeArg, AttributeValue, Bm25, Field, FilterCall, Function, Index,
    IndexKind, Schema, SchemaItem, Table,
};

use super::naming::{pascal_to_snake, surql_default, surql_type};

pub fn emit_schema(schema: &Schema) -> String {
    let mut analyzers = Vec::new();
    let mut functions = Vec::new();
    let mut tables = Vec::new();
    for item in &schema.items {
        match item {
            SchemaItem::AnalyzerDecl(a) => analyzers.push(a),
            SchemaItem::FunctionDecl(f) => functions.push(f),
            SchemaItem::TableDecl(t) => tables.push(t),
            SchemaItem::DocComment { .. } => {}
        }
    }
    analyzers.sort_by(|a, b| a.name.cmp(&b.name));
    functions.sort_by(|a, b| a.name.cmp(&b.name));
    tables.sort_by(|a, b| a.name.cmp(&b.name));

    let mut parts = Vec::new();
    for a in &analyzers {
        parts.push(emit_analyzer(a));
    }
    for f in &functions {
        parts.push(emit_function(f));
    }
    for table in &tables {
        parts.push(emit_table(table));
    }
    for table in &tables {
        let mut fields = table.fields.iter().collect::<Vec<_>>();
        fields.sort_by(|a, b| a.name.cmp(&b.name));
        for field in fields {
            parts.push(emit_field(&table.name, field));
        }
        let mut indexes = table.indexes.iter().collect::<Vec<_>>();
        indexes.sort_by(|a, b| a.name.cmp(&b.name));
        for idx in indexes {
            parts.push(emit_index(&table.name, idx));
        }
    }

    join_statements(parts)
}

pub fn emit_function(f: &Function) -> String {
    let params = f
        .params
        .iter()
        .map(|p| format!("${}: {}", p.name, surql_type(&p.ty)))
        .collect::<Vec<_>>()
        .join(", ");
    let body = f.body.body.trim();
    let permissions = function_permissions(&f.raw_attributes).unwrap_or_else(|| "FULL".to_string());

    format!(
        "DEFINE FUNCTION fn::{}({}) -> {} {{ {} }} PERMISSIONS {};",
        f.name,
        params,
        surql_type(&f.return_type),
        body,
        permissions
    )
}

fn function_permissions(attrs: &[Attribute]) -> Option<String> {
    // TODO: Revisit function attribute handling when the semantic engine owns
    // function-specific lowering/cleanup instead of searching raw attributes here.
    attrs
        .iter()
        .find(|attr| attr.name == "allow")
        .and_then(function_permission_body)
}

fn function_permission_body(attr: &Attribute) -> Option<String> {
    attr.args.iter().find_map(|arg| match arg {
        AttributeArg::Positional {
            value: AttributeValue::Surql { body, .. },
        } => Some(body.trim().to_string()),
        _ => None,
    })
}

pub fn emit_table(t: &Table) -> String {
    let name = pascal_to_snake(&t.name);
    match t.modifier.as_deref() {
        None => format!("DEFINE TABLE {name};"),
        Some("drop") => format!("DEFINE TABLE {name} DROP;"),
        Some("schemafull") => format!("DEFINE TABLE {name} SCHEMAFULL;"),
        Some("schemaless") => format!("DEFINE TABLE {name} SCHEMALESS;"),
        Some(other) => format!("DEFINE TABLE {name} {};", other.to_ascii_uppercase()),
    }
}

pub fn emit_field(table_name: &str, f: &Field) -> String {
    let table_name = pascal_to_snake(table_name);
    let clause = field_clause(f);
    format!("DEFINE FIELD {} ON {} {};", f.name, table_name, clause)
}

pub fn emit_alter_field(table_name: &str, f: &Field) -> String {
    let table_name = pascal_to_snake(table_name);
    let clause = field_clause(f);
    format!("ALTER FIELD {} ON {} {};", f.name, table_name, clause)
}

fn field_clause(f: &Field) -> String {
    let clauses = [field_type_clause(f), field_default_clause(f)];
    clauses
        .into_iter()
        .filter(|clause| !clause.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// `TYPE <ty>` or `TYPE option<...>`, with trailing ` FLEXIBLE` when the field
/// is `object @flexible`.
fn field_type_clause(f: &Field) -> String {
    let ty = surql_type(&f.ty);
    let mut clause = if f.optional {
        format!("TYPE option<{}>", ty)
    } else {
        format!("TYPE {}", ty)
    };
    if f.flexible {
        clause.push_str(" FLEXIBLE");
    }
    clause
}

fn field_default_clause(f: &Field) -> String {
    let Some(default) = &f.default else {
        return String::new();
    };
    let mut clause = "DEFAULT".to_string();
    if f.always {
        clause.push_str(" ALWAYS");
    }
    clause.push_str(&format!(" {}", surql_default(default)));
    clause
}

pub fn emit_remove_field(table_name: &str, field_name: &str) -> String {
    format!(
        "REMOVE FIELD {} ON TABLE {};",
        field_name,
        pascal_to_snake(table_name)
    )
}

pub fn emit_remove_table(table_name: &str) -> String {
    format!("REMOVE TABLE {};", pascal_to_snake(table_name))
}

pub fn emit_analyzer(a: &Analyzer) -> String {
    let mut sql = format!("DEFINE ANALYZER {}", a.name);
    if !a.tokenizers.is_empty() {
        sql.push_str(&format!(" TOKENIZERS {}", a.tokenizers.join(",")));
    }
    if !a.filters.is_empty() {
        let parts: Vec<String> = a.filters.iter().map(filter_call_to_string).collect();
        sql.push_str(&format!(" FILTERS {}", parts.join(",")));
    }
    sql.push(';');
    sql
}

pub fn emit_remove_analyzer(name: &str) -> String {
    format!("REMOVE ANALYZER {};", name)
}

pub fn emit_index(table_name: &str, idx: &Index) -> String {
    let table = pascal_to_snake(table_name);
    let mut sql = format!("DEFINE INDEX {} ON {}", idx.name, table);

    // COUNT indexes don't accept a FIELDS clause; everything else needs one.
    if !matches!(idx.kind, IndexKind::Count) {
        sql.push_str(&format!(" FIELDS {}", idx.fields.join(", ")));
    }

    match &idx.kind {
        IndexKind::Standard => {}
        IndexKind::Unique => sql.push_str(" UNIQUE"),
        IndexKind::Count => sql.push_str(" COUNT"),
        IndexKind::Fulltext {
            analyzer,
            bm25,
            highlights,
        } => {
            sql.push_str(&format!(" FULLTEXT ANALYZER {}", analyzer));
            if let Some(b) = bm25 {
                sql.push_str(&format!(
                    " BM25({}, {})",
                    format_number(b.k1),
                    format_number(b.b)
                ));
            }
            if *highlights {
                sql.push_str(" HIGHLIGHTS");
            }
        }
        IndexKind::Hnsw {
            dimension,
            dist,
            ty,
            efc,
            m,
        } => {
            sql.push_str(&format!(" HNSW DIMENSION {}", dimension));
            if let Some(t) = ty {
                sql.push_str(&format!(" TYPE {}", t.to_ascii_uppercase()));
            }
            if let Some(d) = dist {
                sql.push_str(&format!(" DIST {}", d.to_ascii_uppercase()));
            }
            if let Some(e) = efc {
                sql.push_str(&format!(" EFC {}", e));
            }
            if let Some(mm) = m {
                sql.push_str(&format!(" M {}", mm));
            }
        }
    }
    sql.push(';');
    sql
}

pub fn emit_remove_index(table_name: &str, index_name: &str) -> String {
    format!(
        "REMOVE INDEX {} ON TABLE {};",
        index_name,
        pascal_to_snake(table_name)
    )
}

fn filter_call_to_string(f: &FilterCall) -> String {
    if f.args.is_empty() {
        f.name.clone()
    } else {
        format!("{}({})", f.name, f.args.join(","))
    }
}

#[allow(dead_code)]
fn bm25_to_string(b: &Bm25) -> String {
    format!("BM25({}, {})", format_number(b.k1), format_number(b.b))
}

fn format_number(n: f64) -> String {
    // Render whole numbers without a trailing `.0` so output matches the
    // typical SurrealQL hand-written form (`BM25(1.2, 0.75)` not `BM25(1.2,
    // 0.7500000)`). Drop trailing zeros after a decimal point.
    let s = format!("{}", n);
    s
}

fn join_statements(parts: Vec<String>) -> String {
    if parts.is_empty() {
        String::new()
    } else {
        format!("{}\n", parts.join("\n"))
    }
}
