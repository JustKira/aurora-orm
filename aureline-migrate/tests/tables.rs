#[macro_use]
mod common;

use aureline_migrate::diff::diff_schemas;
use aureline_migrate::ops::Op;
use aureline_migrate::render::emit_up;

use common::{field, schema, table};

#[test]
fn drop_modifier_is_a_table_mode_not_table_removal() {
    let prev = schema(vec![table("Review", None, vec![])]);
    let next = schema(vec![table(
        "Review",
        Some("drop"),
        vec![field("rating", "float", false)],
    )]);

    let ops = diff_schemas(&prev, &next);
    assert!(matches!(
        &ops[..],
        [Op::ChangeTableMode {
            table,
            from: None,
            to: Some(to),
        }, Op::AddField { table: field_table, .. }]
            if table == "Review" && to == "drop" && field_table == "Review"
    ));
    assert_eq!(
        emit_up(&[Op::CreateTable(table(
            "Review",
            Some("drop"),
            vec![field("rating", "float", false)]
        ))]),
        expected_surql!(
            "DEFINE TABLE review DROP;",
            "DEFINE FIELD rating ON review TYPE float;",
        )
    );
}
