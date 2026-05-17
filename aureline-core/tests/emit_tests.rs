use aureline_core::ast::{
    Attribute, AttributeArg, AttributeValue, Function, FunctionParam, Schema, SchemaItem,
    SurqlBlock, Type,
};
use aureline_core::emit::{
    emit_field, emit_function, emit_remove_field, emit_remove_table, emit_schema, emit_table,
    pascal_to_snake,
};
use aureline_test_support::{field, table};

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
fn emits_function_declarations() {
    let function = Function {
        name: "current_user".to_string(),
        source_range: None,
        name_range: None,
        params: vec![FunctionParam {
            name: "fallback".to_string(),
            name_range: None,
            ty: Type::primitive("string"),
        }],
        return_type: Type::Record {
            table: Some("user".to_string()),
        },
        body: SurqlBlock {
            body: "RETURN $auth.id ?? $fallback".to_string(),
        },
        raw_attributes: vec![Attribute {
            name: "allow".to_string(),
            args: vec![
                AttributeArg::Keyword {
                    name: "op".to_string(),
                    value: AttributeValue::String {
                        value: "RUN".to_string(),
                    },
                },
                AttributeArg::Positional {
                    value: AttributeValue::Surql {
                        body: "WHERE $auth.admin = true".to_string(),
                        source_range: None,
                    },
                },
            ],
            source_range: None,
        }],
    };

    let emitted = emit_function(&function);

    assert_eq!(
        emitted,
        "DEFINE FUNCTION fn::current_user($fallback: string) -> record<user> { RETURN $auth.id ?? $fallback } PERMISSIONS WHERE $auth.admin = true;"
    );
}

#[test]
fn emits_function_permissions_full_by_default() {
    let function = Function {
        name: "ping".to_string(),
        source_range: None,
        name_range: None,
        params: vec![],
        return_type: Type::primitive("string"),
        body: SurqlBlock {
            body: "RETURN 'pong'".to_string(),
        },
        raw_attributes: vec![],
    };

    assert_eq!(
        emit_function(&function),
        "DEFINE FUNCTION fn::ping() -> string { RETURN 'pong' } PERMISSIONS FULL;"
    );
}

#[test]
fn emits_schema_deterministically() {
    let function = || Function {
        name: "ping".to_string(),
        source_range: None,
        name_range: None,
        params: vec![],
        return_type: Type::primitive("string"),
        body: SurqlBlock {
            body: "RETURN 'pong'".to_string(),
        },
        raw_attributes: vec![],
    };

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
            SchemaItem::FunctionDecl(function()),
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
            SchemaItem::FunctionDecl(function()),
        ],
    };

    let expected = "DEFINE FUNCTION fn::ping() -> string { RETURN 'pong' } PERMISSIONS FULL;\nDEFINE TABLE temp_log DROP;\nDEFINE TABLE user SCHEMAFULL;\nDEFINE FIELD x ON temp_log TYPE string;\nDEFINE FIELD a ON user TYPE option<string>;\nDEFINE FIELD b ON user TYPE int;\n";
    assert_eq!(emit_schema(&a), expected);
    assert_eq!(emit_schema(&b), expected);
}
