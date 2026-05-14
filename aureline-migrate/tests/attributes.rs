mod common;

use aureline_core::ast::{Schema, SchemaItem};
use aureline_migrate::diff::diff_schemas;
use aureline_migrate::render::emit_up;
use aureline_migrate::snapshot::{canonicalize, parse_snapshot};

#[test]
fn table_field_slice_ignores_index_attributes_for_now() {
    let parsed = aureline_core::parse_validated(
        r#"
table User {
  email string @unique
}
"#,
    )
    .unwrap();

    let ops = diff_schemas(&Schema { items: Vec::new() }, &parsed);
    assert_eq!(
        emit_up(&ops),
        "\
DEFINE TABLE user;
DEFINE FIELD email ON user TYPE string;
"
    );

    let snapshot = parse_snapshot(&canonicalize(&parsed)).unwrap();
    let SchemaItem::TableDecl(table) = &snapshot.items[0] else {
        panic!("expected table snapshot");
    };
    assert!(table.indexes.is_empty());
    assert!(table.raw_attributes.is_empty());
    assert!(table.fields[0].raw_attributes.is_empty());
}
