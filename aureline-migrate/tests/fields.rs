#[macro_use]
mod common;

use aureline_core::ast::Field;
use aureline_migrate::change::{Change, FieldChangeSet};
use aureline_migrate::diff::{diff_changes, diff_schemas};
use aureline_migrate::ops::Op;
use aureline_migrate::render::{emit_down, emit_up};

use common::{field, schema, table};

#[test]
fn diffs_stably_and_marks_destructive_ops() {
    let prev = schema(vec![table(
        "User",
        Some("schemafull"),
        vec![
            field("age", "int", true),
            field("email", "string", false),
            field("name", "string", false),
        ],
    )]);
    let next = schema(vec![table(
        "User",
        Some("schemaless"),
        vec![
            field("age", "int", false),
            field("email", "datetime", false),
            field("score", "float", true),
        ],
    )]);

    let ops = diff_schemas(&prev, &next);
    assert_eq!(ops.len(), 5);
    assert!(matches!(ops[0], Op::ChangeTableMode { .. }));
    assert!(matches!(
        ops[1],
        Op::AlterField {
            changes: FieldChangeSet {
                optional_changed: true,
                ..
            },
            to: Field {
                optional: false,
                ..
            },
            ..
        }
    ));
    assert!(matches!(
        ops[2],
        Op::AlterField {
            changes: FieldChangeSet {
                type_changed: true,
                ..
            },
            ..
        }
    ));
    assert!(matches!(ops[3], Op::RemoveField { .. }));
    assert!(matches!(ops[4], Op::AddField { .. }));
    assert_eq!(ops.iter().filter(|op| op.destructive()).count(), 3);
}

#[test]
fn render_up_and_down() {
    let ops = vec![
        Op::CreateTable(table(
            "User",
            Some("schemafull"),
            vec![field("email", "string", false)],
        )),
        Op::AddField {
            table: "User".to_string(),
            field: field("score", "float", true),
        },
        Op::RemoveField {
            table: "User".to_string(),
            field: field("legacy", "string", false),
        },
    ];

    assert_eq!(
        emit_up(&ops),
        expected_surql!(
            "DEFINE TABLE user SCHEMAFULL;",
            "DEFINE FIELD email ON user TYPE string;",
            "DEFINE FIELD score ON user TYPE option<float>;",
            "REMOVE FIELD legacy ON TABLE user;",
        )
    );
    assert_eq!(
        emit_down(&ops),
        expected_surql!(
            "-- down: RemoveField User.legacy cannot restore data",
            "REMOVE FIELD score ON TABLE user;",
            "REMOVE TABLE user;",
        )
    );
}

#[test]
fn combined_type_and_optional_change_stays_one_op_but_renders_new_optionality() {
    let prev = schema(vec![table("User", None, vec![field("age", "int", false)])]);
    let next = schema(vec![table(
        "User",
        None,
        vec![field("age", "string", true)],
    )]);

    let ops = diff_schemas(&prev, &next);
    assert_eq!(ops.len(), 1);
    assert!(matches!(
        ops[0],
        Op::AlterField {
            changes: FieldChangeSet {
                type_changed: true,
                optional_changed: true,
                ..
            },
            ..
        }
    ));
    assert_eq!(
        emit_up(&ops),
        expected_surql!("ALTER FIELD age ON user TYPE option<string>;")
    );
    assert_eq!(
        emit_down(&ops),
        expected_surql!("ALTER FIELD age ON user TYPE int;")
    );
}

#[test]
fn diff_detects_tables_and_fields_before_planning() {
    let prev = schema(vec![table(
        "User",
        None,
        vec![field("email", "string", false)],
    )]);
    let next = schema(vec![table(
        "User",
        Some("schemafull"),
        vec![
            field("email", "string", true),
            field("name", "string", false),
        ],
    )]);

    let changes = diff_changes(&prev, &next);

    assert_eq!(changes.len(), 3);
    assert!(matches!(changes[0], Change::TableModeChanged { .. }));
    assert!(matches!(
        changes[1],
        Change::FieldChanged {
            changes: FieldChangeSet {
                optional_changed: true,
                ..
            },
            ..
        }
    ));
    assert!(matches!(changes[2], Change::FieldAdded { .. }));
}
