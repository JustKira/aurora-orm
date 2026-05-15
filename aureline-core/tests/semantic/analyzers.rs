// Analyzer tests cover symbols used by full-text indexes. The parser can read
// `@fulltext(analyzer: search)` as syntax, but semantics must verify that the
// analyzer declaration exists somewhere in the schema.

use aureline_core::ast::SchemaItem;
use aureline_test_support::aureline_schema;

use super::common::{assert_no_semantic_errors, assert_semantic_error_contains, checked_schema};

#[test]
fn analyzer_declaration_is_preserved_in_checked_schema() {
    let schema = checked_schema(aureline_schema!(
        "analyzer edu {",
        "  tokenizers blank, class",
        "  filters lowercase, snowball(english)",
        "}",
    ));

    let analyzer = schema
        .items
        .iter()
        .find_map(|item| match item {
            SchemaItem::AnalyzerDecl(analyzer) => Some(analyzer),
            _ => None,
        })
        .expect("analyzer should be present");

    assert_eq!(analyzer.name, "edu");
    assert_eq!(analyzer.tokenizers, vec!["blank", "class"]);
    assert_eq!(analyzer.filters.len(), 2);
    assert_eq!(analyzer.filters[0].name, "lowercase");
    assert!(analyzer.filters[0].args.is_empty());
    assert_eq!(analyzer.filters[1].name, "snowball");
    assert_eq!(analyzer.filters[1].args, vec!["english"]);
}

#[test]
fn fulltext_can_reference_declared_analyzer() {
    assert_no_semantic_errors(aureline_schema!(
        "analyzer search {",
        "  tokenizers blank",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string @fulltext(analyzer: search)",
        "}",
    ));
}

#[test]
fn fulltext_can_reference_analyzer_declared_later() {
    // Analyzer lookup is schema-wide, not order-dependent. This prevents
    // declaration order from leaking into user-facing semantics.
    assert_no_semantic_errors(aureline_schema!(
        "table Article {",
        "  body string @fulltext(analyzer: search)",
        "}",
        "",
        "analyzer search {",
        "  tokenizers blank",
        "  filters lowercase",
        "}",
    ));
}

#[test]
fn fulltext_rejects_unknown_analyzer_reference() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table Article {",
            "  body string @fulltext(analyzer: missing)",
            "}",
        ),
        "unknown analyzer `missing`",
    );
}
