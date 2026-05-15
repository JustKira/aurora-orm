// Full-text tests cover the specialized index form that depends on string
// fields and analyzer symbols. Analyzer existence itself is tested in
// `analyzers.rs`; this file focuses on full-text option semantics.

use aureline_core::ast::IndexKind;
use aureline_test_support::aureline_schema;

use super::common::{assert_semantic_error_contains, checked_schema, table};

#[test]
fn fulltext_with_bm25_tuple_and_highlights_lowers_to_index() {
    let schema = checked_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string @fulltext(analyzer: simple, bm25: (1.2, 0.75), highlights: true)",
        "}",
    ));
    let table = table(&schema, "Article");

    match &table.indexes[0].kind {
        IndexKind::Fulltext {
            analyzer,
            bm25,
            highlights,
        } => {
            assert_eq!(analyzer, "simple");
            let bm25 = bm25.as_ref().expect("bm25 should be set");
            assert!((bm25.k1 - 1.2).abs() < f64::EPSILON);
            assert!((bm25.b - 0.75).abs() < f64::EPSILON);
            assert!(*highlights);
        }
        other => panic!("expected fulltext index, got {other:?}"),
    }
}

#[test]
fn fulltext_accepts_name_keyword_and_surrealdb_default_bm25() {
    // Omitting `bm25:` means Aureline should not emit explicit BM25 values;
    // SurrealDB will apply its own defaults.
    let schema = checked_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank",
        "}",
        "",
        "table Article {",
        "  body string @fulltext(analyzer: simple, name: body_search)",
        "}",
    ));
    let table = table(&schema, "Article");

    assert_eq!(table.indexes[0].name, "body_search");
    match &table.indexes[0].kind {
        IndexKind::Fulltext {
            analyzer,
            bm25,
            highlights,
        } => {
            assert_eq!(analyzer, "simple");
            assert!(bm25.is_none());
            assert!(!*highlights);
        }
        other => panic!("expected fulltext index, got {other:?}"),
    }
}

#[test]
fn fulltext_requires_string_field_and_explicit_analyzer() {
    assert_semantic_error_contains(
        aureline_schema!("table Article {", "  body string @fulltext", "}",),
        "@fulltext requires an `analyzer:` argument",
    );
    assert_semantic_error_contains(
        aureline_schema!(
            "table Article {",
            "  body bytes @fulltext(analyzer: simple)",
            "}",
        ),
        "requires `string`",
    );
}

#[test]
fn fulltext_bm25_must_be_two_numbers() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table Article {",
            "  body string @fulltext(analyzer: simple, bm25: (1.2))",
            "}",
        ),
        "exactly two floats",
    );
    assert_semantic_error_contains(
        aureline_schema!(
            "table Article {",
            "  body string @fulltext(analyzer: simple, bm25: (cosine, f32))",
            "}",
        ),
        "bm25 k1: expected a number",
    );
}

#[test]
fn fulltext_highlights_must_be_bool() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table Article {",
            "  body string @fulltext(analyzer: simple, highlights: yes)",
            "}",
        ),
        "@fulltext highlights: expected bool",
    );
}

#[test]
fn fulltext_is_field_level_only() {
    // Aureline intentionally models full-text as a single-field annotation,
    // even though other index kinds can be table-level.
    assert_semantic_error_contains(
        aureline_schema!(
            "table Article {",
            "  body string",
            "",
            "  @@fulltext(fields: [body])",
            "}",
        ),
        "unknown block attribute `@@fulltext`",
    );
}
