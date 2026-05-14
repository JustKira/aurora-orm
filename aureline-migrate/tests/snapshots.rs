mod common;

use aureline_core::ast::{Schema, SchemaItem};
use aureline_migrate::error::Error;
use aureline_migrate::snapshot::{canonicalize, parse_snapshot};

use common::{field, schema, table};

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
