use aurora_core::*;

#[test]
fn parses_basic_schema() {
    let source = r#"
/// Users table
table User schemafull {
  name    string
  email   string
  age     int
  active  bool
  score   float?
}

table TempLog drop {}
"#;

    let schema = parse_to_ast(source).unwrap();
    assert_eq!(schema.items.len(), 3);

    match &schema.items[0] {
        SchemaItem::DocComment { text } => assert_eq!(text, "Users table"),
        _ => panic!("expected doc comment"),
    }

    match &schema.items[1] {
        SchemaItem::TableDecl(table) => {
            assert_eq!(table.name, "User");
            assert_eq!(table.modifier.as_deref(), Some("schemafull"));
            assert_eq!(table.fields.len(), 5);
            assert_eq!(table.fields[0].name, "name");
            assert_eq!(
                table.fields[0].ty,
                aurora_core::ast::Type::primitive("string")
            );
            assert!(!table.fields[0].optional);
            assert_eq!(table.fields[4].name, "score");
            assert!(table.fields[4].optional);
        }
        _ => panic!("expected table"),
    }

    match &schema.items[2] {
        SchemaItem::TableDecl(table) => {
            assert_eq!(table.name, "TempLog");
            assert_eq!(table.modifier.as_deref(), Some("drop"));
            assert!(table.fields.is_empty());
        }
        _ => panic!("expected table"),
    }
}

#[test]
fn parses_schemaless_table() {
    let source = r#"
table Config schemaless {
  key   string
  value string
}
"#;

    let schema = parse_to_ast(source).unwrap();

    match &schema.items[0] {
        SchemaItem::TableDecl(table) => {
            assert_eq!(table.modifier.as_deref(), Some("schemaless"));
            assert_eq!(table.fields.len(), 2);
        }
        _ => panic!("expected table"),
    }
}

#[test]
fn parses_table_without_modifier() {
    let schema = parse_to_ast("table Bare { name string }").unwrap();

    match &schema.items[0] {
        SchemaItem::TableDecl(table) => {
            assert_eq!(table.name, "Bare");
            assert!(table.modifier.is_none());
            assert_eq!(table.fields.len(), 1);
        }
        _ => panic!("expected table"),
    }
}

#[test]
fn rejects_invalid_schema() {
    assert!(parse_to_ast("table { }").is_err());
}

#[test]
fn emits_json_ast() {
    let json = parse_to_json("table T schemafull { x int }").unwrap();

    assert!(json.contains("\"name\": \"T\""));
    assert!(json.contains("\"kind\": \"primitive\""));
    assert!(json.contains("\"name\": \"int\""));
}

#[test]
fn parse_schema_matches_parse_to_ast() {
    let source = "table T schemafull { x int }";

    assert_eq!(parse_schema(source).unwrap(), parse_to_ast(source).unwrap());
}
