use aureline_core::emit::emit_schema;
use aureline_test_support::{aureline_schema, expected_surql, parse_schema};

use super::assert_validation_contains;

#[test]
fn emits_fulltext_index_with_explicit_analyzer_options() {
    let schema = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string @fulltext(analyzer: simple, bm25: (1.2, 0.75), highlights: true)",
        "}",
    ));

    assert_eq!(
        emit_schema(&schema),
        expected_surql!(
            "DEFINE ANALYZER simple TOKENIZERS blank,class FILTERS lowercase;",
            "DEFINE TABLE article;",
            "DEFINE FIELD body ON article TYPE string;",
            "DEFINE INDEX article_body_fts ON article FIELDS body FULLTEXT ANALYZER simple BM25(1.2, 0.75) HIGHLIGHTS;",
        )
    );
}

#[test]
fn fulltext_is_field_level_only() {
    assert_validation_contains(
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

#[test]
fn fulltext_requires_string_and_explicit_analyzer() {
    assert_validation_contains(
        aureline_schema!("table Article {", "  body string @fulltext", "}",),
        "analyzer",
    );
    assert_validation_contains(
        aureline_schema!(
            "table Article {",
            "  body bytes @fulltext(analyzer: simple)",
            "}",
        ),
        "requires `string`",
    );
}
