use aurora_core::ast::{SchemaItem, Type};
use aurora_core::emit::{emit_field, emit_schema, surql_type};
use aurora_core::parse_to_ast;

/// Helper: parse a single-field table and return the field's AST.
fn parse_field_type(src: &str) -> (Type, bool) {
    let source = format!("table T schemafull {{ x {src} }}");
    let schema = parse_to_ast(&source).expect("parses");
    match schema.items.into_iter().next().expect("has table") {
        SchemaItem::TableDecl(table) => {
            let field = table.fields.into_iter().next().expect("has field");
            (field.ty, field.optional)
        }
        _ => panic!("expected table"),
    }
}

#[test]
fn parses_each_new_primitive() {
    for prim in [
        "decimal", "number", "duration", "uuid", "bytes", "any", "regex", "object", "range",
    ] {
        let (ty, optional) = parse_field_type(prim);
        assert_eq!(ty, Type::primitive(prim));
        assert!(!optional);
    }
}

#[test]
fn parses_existing_primitives() {
    for prim in ["bool", "int", "float", "string", "datetime"] {
        let (ty, optional) = parse_field_type(prim);
        assert_eq!(ty, Type::primitive(prim));
        assert!(!optional);
    }
}

#[test]
fn parses_optional_via_question_mark() {
    let (ty, optional) = parse_field_type("int?");
    assert_eq!(ty, Type::primitive("int"));
    assert!(optional);
}

#[test]
fn parses_optional_via_explicit_option() {
    // option<int> at the top level normalizes to optional=true with ty=int —
    // identical to writing `int?`.
    let (ty, optional) = parse_field_type("option<int>");
    assert_eq!(ty, Type::primitive("int"));
    assert!(optional);
}

#[test]
fn parses_array_unbounded_and_sized() {
    let (ty, _) = parse_field_type("array<string>");
    assert_eq!(
        ty,
        Type::Array {
            inner: Box::new(Type::primitive("string")),
            length: None,
        }
    );

    let (ty, _) = parse_field_type("array<int, 5>");
    assert_eq!(
        ty,
        Type::Array {
            inner: Box::new(Type::primitive("int")),
            length: Some(5),
        }
    );
}

#[test]
fn parses_set() {
    let (ty, _) = parse_field_type("set<uuid>");
    assert_eq!(
        ty,
        Type::Set {
            inner: Box::new(Type::primitive("uuid")),
            length: None,
        }
    );
}

#[test]
fn parses_record_with_and_without_table() {
    let (ty, _) = parse_field_type("record");
    assert_eq!(ty, Type::Record { table: None });

    let (ty, _) = parse_field_type("record<user>");
    assert_eq!(
        ty,
        Type::Record {
            table: Some("user".to_string())
        }
    );
}

#[test]
fn parses_geometry_with_features() {
    let (ty, _) = parse_field_type("geometry<point>");
    assert_eq!(
        ty,
        Type::Geometry {
            features: vec!["point".to_string()],
        }
    );

    let (ty, _) = parse_field_type("geometry<point | polygon | multipolygon>");
    assert_eq!(
        ty,
        Type::Geometry {
            features: vec![
                "point".to_string(),
                "polygon".to_string(),
                "multipolygon".to_string(),
            ],
        }
    );
}

#[test]
fn parses_nested_compound_types() {
    // array<option<string>> — the option here is *nested*, not normalized away.
    let (ty, optional) = parse_field_type("array<option<string>>");
    assert!(!optional);
    assert_eq!(
        ty,
        Type::Array {
            inner: Box::new(Type::Option {
                inner: Box::new(Type::primitive("string")),
            }),
            length: None,
        }
    );
}

#[test]
fn renders_each_type_to_surql() {
    let cases = [
        (Type::primitive("string"), "string"),
        (Type::primitive("decimal"), "decimal"),
        (
            Type::Option {
                inner: Box::new(Type::primitive("int")),
            },
            "option<int>",
        ),
        (
            Type::Array {
                inner: Box::new(Type::primitive("string")),
                length: None,
            },
            "array<string>",
        ),
        (
            Type::Array {
                inner: Box::new(Type::primitive("int")),
                length: Some(3),
            },
            "array<int, 3>",
        ),
        (
            Type::Set {
                inner: Box::new(Type::primitive("uuid")),
                length: None,
            },
            "set<uuid>",
        ),
        (Type::Record { table: None }, "record"),
        (
            Type::Record {
                table: Some("User".to_string()),
            },
            "record<user>",
        ),
        (
            Type::Geometry {
                features: vec!["point".to_string(), "polygon".to_string()],
            },
            "geometry<point | polygon>",
        ),
    ];
    for (ty, expected) in cases {
        assert_eq!(surql_type(&ty), expected, "for type {ty:?}");
    }
}

#[test]
fn emits_field_with_compound_types() {
    use aurora_core::ast::Field;
    let field = Field {
        name: "tags".to_string(),
        ty: Type::Array {
            inner: Box::new(Type::primitive("string")),
            length: None,
        },
        optional: false,
        flexible: false,
        raw_attributes: Vec::new(),
    };
    assert_eq!(
        emit_field("Article", &field),
        "DEFINE FIELD tags ON article TYPE array<string>;"
    );

    let field = Field {
        name: "author".to_string(),
        ty: Type::Record {
            table: Some("User".to_string()),
        },
        optional: true,
        flexible: false,
        raw_attributes: Vec::new(),
    };
    assert_eq!(
        emit_field("Article", &field),
        "DEFINE FIELD author ON article TYPE option<record<user>>;"
    );
}

#[test]
fn full_schema_round_trip() {
    let source = r#"
table Article schemafull {
  title    string
  body     string
  tags     array<string>
  author   record<user>
  views    int?
  geo      geometry<point>
  metadata object
}
"#;

    let schema = parse_to_ast(source).expect("parses");
    let surql = emit_schema(&schema);

    // Spot-check that compound types render correctly. (Order is alphabetical.)
    assert!(surql.contains("DEFINE FIELD author ON article TYPE record<user>;"));
    assert!(surql.contains("DEFINE FIELD tags ON article TYPE array<string>;"));
    assert!(surql.contains("DEFINE FIELD geo ON article TYPE geometry<point>;"));
    assert!(surql.contains("DEFINE FIELD metadata ON article TYPE object;"));
    assert!(surql.contains("DEFINE FIELD views ON article TYPE option<int>;"));
}
