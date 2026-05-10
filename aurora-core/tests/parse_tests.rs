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
fn parse_error_highlights_invalid_type_token() {
    let err = parse_to_ast("table Demo { ttl duratio }").unwrap_err();
    let AuroraError::Parse(diagnostic) = err else {
        panic!("expected parse diagnostic");
    };

    assert_eq!(diagnostic.range.start.line, 0);
    assert_eq!(diagnostic.range.start.character, 17);
    assert_eq!(diagnostic.range.end.character, 24);
    assert!(
        diagnostic.message.contains("type"),
        "{}",
        diagnostic.message
    );
    assert!(
        !diagnostic.message.contains("type_node"),
        "{}",
        diagnostic.message
    );
}

#[test]
fn parse_error_uses_friendly_rule_names() {
    let err = parse_to_ast("table primitives_demo schemaful { }").unwrap_err();
    let AuroraError::Parse(diagnostic) = err else {
        panic!("expected parse diagnostic");
    };

    assert_eq!(diagnostic.range.start.line, 0);
    assert_eq!(diagnostic.range.start.character, 22);
    assert_eq!(diagnostic.range.end.character, 31);
    assert!(
        diagnostic
            .message
            .contains("`schemafull`, `schemaless`, or `drop`"),
        "{}",
        diagnostic.message
    );
    assert!(
        !diagnostic.message.contains("table_modifier"),
        "{}",
        diagnostic.message
    );
}

#[test]
fn parse_error_for_table_header_without_body_reports_missing_brace() {
    let err = parse_to_ast("table primitives_demo schemafull ").unwrap_err();
    let AuroraError::Parse(diagnostic) = err else {
        panic!("expected parse diagnostic");
    };

    assert_eq!(diagnostic.message, "expected `{` to start table body");
    assert_eq!(diagnostic.range.start.line, 0);
    assert_eq!(diagnostic.range.start.character, 33);
}

#[test]
fn parse_error_for_analyzer_header_without_body_reports_missing_brace() {
    let err = parse_to_ast("analyzer edu ").unwrap_err();
    let AuroraError::Parse(diagnostic) = err else {
        panic!("expected parse diagnostic");
    };

    assert_eq!(diagnostic.message, "expected `{` to start analyzer body");
    assert_eq!(diagnostic.range.start.line, 0);
    assert_eq!(diagnostic.range.start.character, 13);
}

#[test]
fn check_suggests_top_level_declaration_keyword_from_recovery_node() {
    let report = aurora_core::check("tabl compound_demo schemafull");
    let diagnostic = report.diagnostics.first().expect("diagnostic");

    assert!(report.schema.is_some());
    assert_eq!(
        diagnostic.message,
        "unknown top-level declaration `tabl`; did you mean `table`?"
    );
    assert_eq!(diagnostic.range.start.line, 0);
    assert_eq!(diagnostic.range.start.character, 0);
    assert_eq!(diagnostic.range.end.character, 29);
}

#[test]
fn strict_parse_rejects_unknown_top_level_declaration() {
    let err = parse_to_ast("tabl compound_demo schemafull").unwrap_err();
    let AuroraError::Parse(diagnostic) = err else {
        panic!("expected parse diagnostic");
    };

    assert!(diagnostic.message.contains("expected"));
    assert_eq!(diagnostic.range.start.line, 0);
    assert_eq!(diagnostic.range.start.character, 0);
    assert_eq!(diagnostic.range.end.character, 4);
}

#[test]
fn parse_error_explains_array_length_syntax() {
    let err = parse_to_ast("table Demo { tags array<string, > }").unwrap_err();
    let AuroraError::Parse(diagnostic) = err else {
        panic!("expected parse diagnostic");
    };

    assert!(
        diagnostic.message.contains("array length like `10`"),
        "{}",
        diagnostic.message
    );
    assert!(
        diagnostic.message.contains("`array<T>` or `array<T, N>`"),
        "{}",
        diagnostic.message
    );
}

#[test]
fn parse_error_explains_geometry_type_syntax() {
    let err = parse_to_ast("table Demo { shape geometry<> }").unwrap_err();
    let AuroraError::Parse(diagnostic) = err else {
        panic!("expected parse diagnostic");
    };

    assert!(
        diagnostic.message.contains("geometry feature names"),
        "{}",
        diagnostic.message
    );
    assert!(
        diagnostic.message.contains("`geometry<point>`"),
        "{}",
        diagnostic.message
    );
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
