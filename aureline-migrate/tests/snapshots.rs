#[macro_use]
mod common;

use aureline_core::ast::{Schema, SchemaItem};
use aureline_migrate::error::Error;
use aureline_migrate::snapshot::{canonicalize, parse_snapshot};

use common::{field, parse_schema, schema, table};

#[test]
fn snapshots_are_canonical_and_roundtrip() {
    let a = Schema {
        items: vec![
            SchemaItem::DocComment {
                text: "ignored".to_string(),
            },
            SchemaItem::TableDecl(table(
                "User",
                None,
                vec![field("b", "int", false), field("a", "string", true)],
            )),
        ],
    };
    let b = schema(vec![table(
        "User",
        None,
        vec![field("a", "string", true), field("b", "int", false)],
    )]);

    let canonical = canonicalize(&a);
    assert_eq!(canonical, canonicalize(&b));
    assert_eq!(parse_snapshot(&canonical).unwrap(), b);
    assert!(matches!(
        parse_snapshot(r#"{"version":2,"tables":[]}"#),
        Err(Error::SnapshotDecode(_))
    ));
}

#[test]
fn snapshots_preserve_parser_lowered_analyzers_and_indexes() {
    let parsed = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  account string",
        "  body string @fulltext(analyzer: simple, highlights: true)",
        "  embedding array<float> @hnsw(dimension: 1536, dist: cosine)",
        "  email string @unique",
        "  status string @index",
        "",
        "  @@index(fields: [account, status], name: account_status_lookup)",
        "  @@count",
        "}",
    ));

    let canonical = canonicalize(&parsed);
    let roundtrip = parse_snapshot(&canonical).unwrap();

    assert_eq!(canonicalize(&roundtrip), canonical);
    assert!(canonical.contains("simple"));
    assert!(canonical.contains("article_body_fts"));
    assert!(canonical.contains("article_embedding_hnsw"));
    assert!(canonical.contains("article_count"));
}
