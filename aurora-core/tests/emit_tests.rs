use aurora_core::ast::{Field, Schema, SchemaItem, Table, Type};
use aurora_core::emit::{
    emit_field, emit_remove_field, emit_remove_table, emit_schema, emit_surql_block, emit_table,
    pascal_to_snake,
};

#[test]
fn converts_pascal_to_snake() {
    assert_eq!(pascal_to_snake("User"), "user");
    assert_eq!(pascal_to_snake("TempLog"), "temp_log");
    assert_eq!(pascal_to_snake("UserProfile"), "user_profile");
    assert_eq!(pascal_to_snake("Config"), "config");
    assert_eq!(pascal_to_snake("X"), "x");
    assert_eq!(pascal_to_snake("HTTPRequest"), "httprequest");
}

#[test]
fn emits_tables_fields_and_removes() {
    assert_eq!(
        emit_table(&table("User", Some("schemafull"), vec![])),
        "DEFINE TABLE user SCHEMAFULL;"
    );
    assert_eq!(
        emit_table(&table("UserProfile", Some("schemaless"), vec![])),
        "DEFINE TABLE user_profile SCHEMALESS;"
    );
    assert_eq!(
        emit_table(&table("TempLog", Some("drop"), vec![])),
        "DEFINE TABLE temp_log DROP;"
    );
    assert_eq!(
        emit_table(&table("Audit", Some("flex"), vec![])),
        "DEFINE TABLE audit FLEX;"
    );
    assert_eq!(
        emit_field("UserProfile", &field("email", "string", false)),
        "DEFINE FIELD email ON user_profile TYPE string;"
    );
    assert_eq!(
        emit_field("UserProfile", &field("score", "float", true)),
        "DEFINE FIELD score ON user_profile TYPE option<float>;"
    );
    assert_eq!(
        emit_remove_field("UserProfile", "score"),
        "REMOVE FIELD score ON TABLE user_profile;"
    );
    assert_eq!(
        emit_remove_table("UserProfile"),
        "REMOVE TABLE user_profile;"
    );
}

#[test]
fn emits_schema_deterministically() {
    let a = Schema {
        items: vec![
            SchemaItem::TableDecl(table(
                "User",
                Some("schemafull"),
                vec![field("b", "int", false), field("a", "string", true)],
            )),
            SchemaItem::DocComment {
                text: "ignored".to_string(),
            },
            SchemaItem::TableDecl(table(
                "TempLog",
                Some("drop"),
                vec![field("x", "string", false)],
            )),
        ],
    };
    let b = Schema {
        items: vec![
            SchemaItem::TableDecl(table(
                "TempLog",
                Some("drop"),
                vec![field("x", "string", false)],
            )),
            SchemaItem::TableDecl(table(
                "User",
                Some("schemafull"),
                vec![field("a", "string", true), field("b", "int", false)],
            )),
        ],
    };

    let expected = "DEFINE TABLE temp_log DROP;\nDEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD x ON temp_log TYPE string;\nDEFINE FIELD a ON user TYPE option<string>;\nDEFINE FIELD b ON user TYPE int;\n";
    assert_eq!(emit_schema(&a), expected);
    assert_eq!(emit_schema(&b), expected);
}

#[test]
fn emits_raw_surql_blocks() {
    let block = aurora_core::ast::SurqlBlock {
        body: "\n  RETURN 1;\n".to_string(),
    };

    assert_eq!(emit_surql_block(&block), "RETURN 1;");
}

#[test]
fn parser_rejects_schema_with_top_level_raw_surql_block() {
    assert!(aurora_core::parse_to_ast("#surql { RETURN 1; }").is_err());
}

fn table(name: &str, modifier: Option<&str>, fields: Vec<Field>) -> Table {
    Table {
        name: name.to_string(),
        modifier: modifier.map(str::to_string),
        fields,
        indexes: Vec::new(),
        raw_attributes: Vec::new(),
    }
}

fn field(name: &str, type_name: &str, optional: bool) -> Field {
    Field {
        name: name.to_string(),
        ty: Type::primitive(type_name),
        optional,
        flexible: false,
        raw_attributes: Vec::new(),
    }
}
