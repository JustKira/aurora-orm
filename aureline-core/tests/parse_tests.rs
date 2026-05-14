use aureline_core::*;

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

table TempLog drop {
}
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
                aureline_core::ast::Type::primitive("string")
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
    let schema = parse_to_ast(
        r#"
table Bare {
  name string
}
"#,
    )
    .unwrap();

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
    let err = parse_to_ast(
        r#"
table Demo {
  ttl duratio
}
"#,
    )
    .unwrap_err();
    let AurelineError::Parse(diagnostic) = err else {
        panic!("expected parse diagnostic");
    };

    assert_eq!(diagnostic.range.start.line, 2);
    assert_eq!(diagnostic.range.start.character, 6);
    assert_eq!(diagnostic.range.end.character, 13);
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
    let err = parse_to_ast("table primitives_demo schemaful {\n}\n").unwrap_err();
    let AurelineError::Parse(diagnostic) = err else {
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
    let AurelineError::Parse(diagnostic) = err else {
        panic!("expected parse diagnostic");
    };

    assert_eq!(diagnostic.message, "expected `{` to start table body");
    assert_eq!(diagnostic.range.start.line, 0);
    assert_eq!(diagnostic.range.start.character, 33);
}

#[test]
fn parse_error_for_analyzer_header_without_body_reports_missing_brace() {
    let err = parse_to_ast("analyzer edu ").unwrap_err();
    let AurelineError::Parse(diagnostic) = err else {
        panic!("expected parse diagnostic");
    };

    assert_eq!(diagnostic.message, "expected `{` to start analyzer body");
    assert_eq!(diagnostic.range.start.line, 0);
    assert_eq!(diagnostic.range.start.character, 13);
}

#[test]
fn check_suggests_source_item_keyword_from_recovery_node() {
    let report = aureline_core::check("tabl compound_demo schemafull");
    let diagnostic = report.diagnostics.first().expect("diagnostic");

    assert!(report.schema.is_some());
    assert_eq!(
        diagnostic.message,
        "unknown source item `tabl`; did you mean `table`?"
    );
    assert_eq!(diagnostic.range.start.line, 0);
    assert_eq!(diagnostic.range.start.character, 0);
    assert_eq!(diagnostic.range.end.character, 29);
}

#[test]
fn strict_parse_rejects_unknown_source_declaration() {
    let err = parse_to_ast("tabl compound_demo schemafull").unwrap_err();
    let AurelineError::Parse(diagnostic) = err else {
        panic!("expected parse diagnostic");
    };

    assert!(diagnostic.message.contains("expected"));
    assert_eq!(diagnostic.range.start.line, 0);
    assert_eq!(diagnostic.range.start.character, 0);
    assert_eq!(diagnostic.range.end.character, 4);
}

#[test]
fn parse_error_explains_array_length_syntax() {
    let err = parse_to_ast(
        r#"
table Demo {
  tags array<string, >
}
"#,
    )
    .unwrap_err();
    let AurelineError::Parse(diagnostic) = err else {
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
    let err = parse_to_ast(
        r#"
table Demo {
  shape geometry<>
}
"#,
    )
    .unwrap_err();
    let AurelineError::Parse(diagnostic) = err else {
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
fn rejects_whitespace_after_attribute_sigils() {
    assert!(parse_to_ast("table User { email string @ index }").is_err());
    assert!(parse_to_ast("table User { @@ index }").is_err());
}

#[test]
fn emits_json_ast() {
    let json = parse_to_json(
        r#"
table T schemafull {
  x int
}
"#,
    )
    .unwrap();

    assert!(json.contains("\"name\": \"T\""));
    assert!(json.contains("\"kind\": \"primitive\""));
    assert!(json.contains("\"name\": \"int\""));
}

#[test]
fn rejects_top_level_surql_block() {
    assert!(parse_to_ast("#surql { RETURN 1; }").is_err());
}

#[test]
fn parses_assert_surql_block_as_raw_attribute() {
    let source = r#"
table User {
  email string @assert(#surql {
    $value != NONE AND string::is::email($value)
  })
}
"#;

    let schema = parse_to_ast(source).unwrap();

    match &schema.items[0] {
        SchemaItem::TableDecl(table) => {
            let attr = &table.fields[0].raw_attributes[0];
            assert_eq!(attr.name, "assert");
            match &attr.args[0] {
                aureline_core::ast::AttributeArg::Positional {
                    value: aureline_core::ast::AttributeValue::Surql { body, .. },
                } => {
                    assert!(body.contains("$value != NONE"));
                    assert!(body.contains("string::is::email"));
                }
                other => panic!("expected SurQL assert arg, got {other:?}"),
            }
        }
        _ => panic!("expected table"),
    }
}

#[test]
fn parses_surql_block_with_brace_inside_string_literal() {
    let source = r#"
table User {
  marker string @assert(#surql { RETURN "}" })
}
"#;

    let schema = parse_to_ast(source).unwrap();

    match &schema.items[0] {
        SchemaItem::TableDecl(table) => match &table.fields[0].raw_attributes[0].args[0] {
            aureline_core::ast::AttributeArg::Positional {
                value: aureline_core::ast::AttributeValue::Surql { body, .. },
            } => assert_eq!(body, r#" RETURN "}" "#),
            other => panic!("expected SurQL assert arg, got {other:?}"),
        },
        _ => panic!("expected table"),
    }
}

#[test]
fn parses_surql_block_with_brace_inside_comment() {
    let source = r#"
table User {
  marker string @assert(#surql {
    // comment with }
    RETURN true
  })
}
"#;

    let schema = parse_to_ast(source).unwrap();

    match &schema.items[0] {
        SchemaItem::TableDecl(table) => match &table.fields[0].raw_attributes[0].args[0] {
            aureline_core::ast::AttributeArg::Positional {
                value: aureline_core::ast::AttributeValue::Surql { body, .. },
            } => {
                assert!(body.contains("// comment with }"));
                assert!(body.contains("RETURN true"));
            }
            other => panic!("expected SurQL assert arg, got {other:?}"),
        },
        _ => panic!("expected table"),
    }
}

#[test]
fn parses_keyword_args_with_surql_escape_hatch() {
    let source = r#"
table User {
  email string @allow(op: "SELECT", #surql { WHERE $auth.id = id })
}
"#;

    let schema = parse_to_ast(source).unwrap();

    match &schema.items[0] {
        SchemaItem::TableDecl(table) => {
            let attr = &table.fields[0].raw_attributes[0];
            assert_eq!(attr.name, "allow");
            assert_eq!(attr.args.len(), 2);
            assert!(matches!(
                &attr.args[0],
                aureline_core::ast::AttributeArg::Keyword {
                    name,
                    value: aureline_core::ast::AttributeValue::String { value },
                } if name == "op" && value == "SELECT"
            ));
            assert!(matches!(
                &attr.args[1],
                aureline_core::ast::AttributeArg::Positional {
                    value: aureline_core::ast::AttributeValue::Surql { body, .. },
                } if body.contains("WHERE $auth.id = id")
            ));
        }
        _ => panic!("expected table"),
    }
}

#[test]
fn parses_inline_surql_shorthand_as_surql_attribute_value() {
    let source = r#"
table User {
  email string @assert(#s`$value != NONE`)
  id string @allow(op: "SELECT", #s`WHERE $auth.id = id`)
}
"#;

    let schema = parse_validated(source).unwrap();

    match &schema.items[0] {
        SchemaItem::TableDecl(table) => {
            let assert_attr = &table.fields[0].raw_attributes[0];
            assert_eq!(assert_attr.name, "assert");
            assert!(matches!(
                &assert_attr.args[0],
                aureline_core::ast::AttributeArg::Positional {
                    value: aureline_core::ast::AttributeValue::Surql { body, .. },
                } if body == "$value != NONE"
            ));

            let allow_attr = &table.fields[1].raw_attributes[0];
            assert_eq!(allow_attr.name, "allow");
            assert!(matches!(
                &allow_attr.args[1],
                aureline_core::ast::AttributeArg::Positional {
                    value: aureline_core::ast::AttributeValue::Surql { body, .. },
                } if body == "WHERE $auth.id = id"
            ));
        }
        _ => panic!("expected table"),
    }
}

#[test]
fn rejects_gap_before_inline_surql_backtick() {
    let source = r#"
table User {
  email string @assert(#s `$value != NONE`)
}
"#;

    assert!(parse_to_ast(source).is_err());
}

#[test]
fn parses_field_attribute_blocks() {
    let source = r#"
table User {
  id string {
    @allow(op: "SELECT", #surql { WHERE $auth.id != NONE })
    @allow(op: "UPDATE",#surql { WHERE $auth.id != NONE })
  }
}
"#;

    let schema = parse_validated(source).unwrap();

    match &schema.items[0] {
        SchemaItem::TableDecl(table) => {
            assert_eq!(table.fields.len(), 1);
            assert_eq!(table.fields[0].raw_attributes.len(), 2);
            assert_eq!(table.fields[0].raw_attributes[0].name, "allow");
            assert_eq!(table.fields[0].raw_attributes[1].name, "allow");
        }
        _ => panic!("expected table"),
    }
}

#[test]
fn emits_json_ast_with_assert_surql_block() {
    let json = parse_to_json(
        r#"
table User {
  email string @assert(#surql { $value != NONE })
}
"#,
    )
    .unwrap();

    assert!(json.contains("\"name\": \"assert\""));
    assert!(json.contains("\"kind\": \"surql\""));
    assert!(json.contains("$value != NONE"));
}

#[test]
fn parse_schema_matches_parse_to_ast() {
    let source = r#"
table T schemafull {
  x int
}
"#;

    assert_eq!(parse_schema(source).unwrap(), parse_to_ast(source).unwrap());
}
