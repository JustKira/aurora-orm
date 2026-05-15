#[macro_use]
mod common;

use common::{diff_down, diff_up, empty_schema, parse_schema};

#[test]
fn adds_analyzer_definition() {
    let next = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
    ));

    assert_eq!(
        diff_up(&empty_schema(), &next),
        expected_surql!("DEFINE ANALYZER simple TOKENIZERS blank,class FILTERS lowercase;")
    );
    assert_eq!(
        diff_down(&empty_schema(), &next),
        expected_surql!("REMOVE ANALYZER simple;")
    );
}

#[test]
fn removes_analyzer_definition() {
    let prev = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &empty_schema()),
        expected_surql!("REMOVE ANALYZER simple;")
    );
    assert_eq!(
        diff_down(&prev, &empty_schema()),
        expected_surql!("DEFINE ANALYZER simple TOKENIZERS blank,class FILTERS lowercase;")
    );
}

#[test]
fn changes_analyzer_by_replacing_definition() {
    let prev = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank",
        "  filters lowercase",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase, snowball(english)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "REMOVE ANALYZER simple;",
            "DEFINE ANALYZER simple TOKENIZERS blank,class FILTERS lowercase,snowball(english);",
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "REMOVE ANALYZER simple;",
            "DEFINE ANALYZER simple TOKENIZERS blank FILTERS lowercase;",
        )
    );
}

#[test]
fn creates_analyzer_before_fulltext_index_that_references_it() {
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
        diff_up(&empty_schema(), &next),
        expected_surql!(
            "DEFINE ANALYZER simple TOKENIZERS blank,class FILTERS lowercase;",
            "DEFINE TABLE article;",
            "DEFINE FIELD body ON article TYPE string;",
            "DEFINE INDEX article_body_fts ON article FIELDS body FULLTEXT ANALYZER simple;",
        )
    );
}
