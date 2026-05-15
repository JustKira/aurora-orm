use super::common::{diff_down, diff_up, empty_schema, parse_schema};

#[test]
fn adds_fulltext_index_with_explicit_analyzer() {
    let prev = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string @fulltext(analyzer: simple)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "DEFINE INDEX article_body_fts ON article FIELDS body FULLTEXT ANALYZER simple;"
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX article_body_fts ON TABLE article;")
    );
}

#[test]
fn adds_fulltext_index_with_bm25_tuple() {
    let prev = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string @fulltext(analyzer: simple, bm25: (1.2, 0.75))",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "DEFINE INDEX article_body_fts ON article FIELDS body FULLTEXT ANALYZER simple BM25(1.2, 0.75);"
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX article_body_fts ON TABLE article;")
    );
}

#[test]
fn adds_fulltext_index_with_highlights() {
    let prev = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string @fulltext(analyzer: simple, highlights: true)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "DEFINE INDEX article_body_fts ON article FIELDS body FULLTEXT ANALYZER simple HIGHLIGHTS;"
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX article_body_fts ON TABLE article;")
    );
}

#[test]
fn adds_fulltext_index_with_bm25_and_highlights() {
    let prev = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
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
        diff_up(&prev, &next),
        expected_surql!(
            "DEFINE INDEX article_body_fts ON article FIELDS body FULLTEXT ANALYZER simple BM25(1.2, 0.75) HIGHLIGHTS;"
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX article_body_fts ON TABLE article;")
    );
}

#[test]
fn creates_table_with_fulltext_index_after_analyzer_and_fields() {
    let next = parse_schema(aureline_schema!(
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
        diff_up(&empty_schema(), &next),
        expected_surql!(
            "DEFINE ANALYZER simple TOKENIZERS blank,class FILTERS lowercase;",
            "DEFINE TABLE article;",
            "DEFINE FIELD body ON article TYPE string;",
            "DEFINE INDEX article_body_fts ON article FIELDS body FULLTEXT ANALYZER simple BM25(1.2, 0.75) HIGHLIGHTS;",
        )
    );
}

#[test]
fn removes_fulltext_index_from_existing_table() {
    let prev = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string @fulltext(analyzer: simple)",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("REMOVE INDEX article_body_fts ON TABLE article;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "DEFINE INDEX article_body_fts ON article FIELDS body FULLTEXT ANALYZER simple;"
        )
    );
}

#[test]
fn changes_fulltext_options_by_replacing_the_index() {
    let prev = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string @fulltext(analyzer: simple, name: body_search)",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string @fulltext(analyzer: simple, bm25: (1.2, 0.75), highlights: true, name: body_search)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "REMOVE INDEX body_search ON TABLE article;",
            "DEFINE INDEX body_search ON article FIELDS body FULLTEXT ANALYZER simple BM25(1.2, 0.75) HIGHLIGHTS;",
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "REMOVE INDEX body_search ON TABLE article;",
            "DEFINE INDEX body_search ON article FIELDS body FULLTEXT ANALYZER simple;",
        )
    );
}
