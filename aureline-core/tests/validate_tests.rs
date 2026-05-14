use aureline_core::ast::{IndexKind, SchemaItem};
use aureline_core::{AurelineError, parse_validated};
use aureline_test_support::extract_table;

#[test]
fn unique_field_annotation_creates_unique_index() {
    let src = r#"
table user {
  email string @unique
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "user");
    assert_eq!(table.indexes.len(), 1);
    assert_eq!(table.indexes[0].name, "user_email_unique");
    assert_eq!(table.indexes[0].fields, vec!["email"]);
    assert!(matches!(table.indexes[0].kind, IndexKind::Unique));
}

#[test]
fn unique_field_annotation_rejects_fields_arg() {
    let src = r#"
table membership {
  account string
  user    string
  account_user string @unique(fields: [account, user])
}
"#;
    let err = parse_validated(src).unwrap_err();
    let AurelineError::Validation(errs) = err else {
        panic!("expected validation error");
    };
    assert_eq!(errs.len(), 1);
    assert!(
        errs[0].message.contains("unknown @unique arg `fields`"),
        "{}",
        errs[0].message
    );
    assert!(errs[0].message.contains("expected `name`"));
}

#[test]
fn index_field_annotation_creates_standard_index() {
    let src = r#"
table user {
  status string @index
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "user");
    assert_eq!(table.indexes.len(), 1);
    assert_eq!(table.indexes[0].name, "user_status_idx");
    assert!(matches!(table.indexes[0].kind, IndexKind::Standard));
}

#[test]
fn flexible_on_object_sets_field_flag() {
    let src = r#"
table doc {
  meta object @flexible
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "doc");
    assert!(table.fields[0].flexible);
}

#[test]
fn flexible_on_non_object_errors() {
    let src = r#"
table doc {
  body string @flexible
}
"#;
    let err = parse_validated(src).unwrap_err();
    let AurelineError::Validation(errs) = err else {
        panic!("expected validation error");
    };
    assert_eq!(errs.len(), 1);
    assert!(errs[0].message.contains("@flexible"), "{}", errs[0].message);
    assert!(
        errs[0].message.contains("requires `object`"),
        "{}",
        errs[0].message
    );
}

#[test]
fn hnsw_keyword_form() {
    let src = r#"
table doc {
  embedding array<float> @hnsw(dimension: 1536, dist: cosine, type: f32, efc: 200, m: 16)
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "doc");
    assert_eq!(table.indexes.len(), 1);
    match &table.indexes[0].kind {
        IndexKind::Hnsw {
            dimension,
            dist,
            ty,
            efc,
            m,
        } => {
            assert_eq!(*dimension, 1536);
            assert_eq!(dist.as_deref(), Some("cosine"));
            assert_eq!(ty.as_deref(), Some("f32"));
            assert_eq!(*efc, Some(200));
            assert_eq!(*m, Some(16));
        }
        other => panic!("expected Hnsw, got {other:?}"),
    }
}

#[test]
fn hnsw_dimension_only() {
    let src = r#"
table doc {
  embedding array<float> @hnsw(dimension: 384)
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "doc");
    match &table.indexes[0].kind {
        IndexKind::Hnsw {
            dimension,
            dist,
            ty,
            ..
        } => {
            assert_eq!(*dimension, 384);
            assert!(dist.is_none());
            assert!(ty.is_none());
        }
        _ => panic!("expected Hnsw"),
    }
}

#[test]
fn hnsw_on_string_errors() {
    let src = r#"
table doc {
  text string @hnsw(dimension: 1536)
}
"#;
    let err = parse_validated(src).unwrap_err();
    let AurelineError::Validation(errs) = err else {
        panic!("expected validation error");
    };
    assert!(errs[0].message.contains("@hnsw"), "{}", errs[0].message);
    assert!(
        errs[0].message.contains("array<float>"),
        "{}",
        errs[0].message
    );
}

#[test]
fn hnsw_missing_dimension_errors() {
    let src = r#"
table doc {
  embedding array<float> @hnsw(dist: cosine)
}
"#;
    let err = parse_validated(src).unwrap_err();
    let AurelineError::Validation(errs) = err else {
        panic!("expected validation error");
    };
    assert!(errs[0].message.contains("dimension"), "{}", errs[0].message);
}

#[test]
fn fulltext_with_bm25_tuple_and_highlights() {
    let src = r#"
analyzer simple {
  tokenizers blank, class
  filters lowercase
}

table article {
  body string @fulltext(analyzer: simple, bm25: (1.2, 0.75), highlights: true)
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "article");
    match &table.indexes[0].kind {
        IndexKind::Fulltext {
            analyzer,
            bm25,
            highlights,
        } => {
            assert_eq!(analyzer, "simple");
            let bm = bm25.as_ref().expect("bm25 set");
            assert!((bm.k1 - 1.2).abs() < 1e-9);
            assert!((bm.b - 0.75).abs() < 1e-9);
            assert!(*highlights);
        }
        _ => panic!("expected Fulltext"),
    }
}

#[test]
fn fulltext_analyzer_only() {
    // Omitting `bm25:` means "use SurrealDB's default BM25 params".
    let src = r#"
table article {
  body string @fulltext(analyzer: simple)
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "article");
    match &table.indexes[0].kind {
        IndexKind::Fulltext {
            analyzer,
            bm25,
            highlights,
        } => {
            assert_eq!(analyzer, "simple");
            assert!(bm25.is_none());
            assert!(!*highlights);
        }
        _ => panic!("expected Fulltext"),
    }
}

#[test]
fn fulltext_requires_explicit_analyzer() {
    let src = r#"
table article {
  body string @fulltext
}
"#;
    let err = parse_validated(src).unwrap_err();
    let AurelineError::Validation(errs) = err else {
        panic!("expected validation error");
    };
    assert!(errs[0].message.contains("analyzer"), "{}", errs[0].message);
}

#[test]
fn fulltext_bm25_tuple_wrong_arity() {
    let src = r#"
table article {
  body string @fulltext(analyzer: simple, bm25: (1.2))
}
"#;
    let err = parse_validated(src).unwrap_err();
    let AurelineError::Validation(errs) = err else {
        panic!("expected validation error");
    };
    assert!(errs[0].message.contains("bm25"), "{}", errs[0].message);
    assert!(errs[0].message.contains("two"), "{}", errs[0].message);
}

#[test]
fn fulltext_bm25_tuple_non_numeric() {
    let src = r#"
table article {
  body string @fulltext(analyzer: simple, bm25: (cosine, f32))
}
"#;
    let err = parse_validated(src).unwrap_err();
    let AurelineError::Validation(errs) = err else {
        panic!("expected validation error");
    };
    assert!(errs[0].message.contains("number"), "{}", errs[0].message);
}

#[test]
fn unknown_attribute_errors_with_suggestion() {
    let src = r#"
table user {
  email string @uniqu
}
"#;
    let err = parse_validated(src).unwrap_err();
    let AurelineError::Validation(errs) = err else {
        panic!("expected validation error");
    };
    assert!(errs[0].message.contains("@uniqu"), "{}", errs[0].message);
    assert_eq!(errs[0].hint.as_deref(), Some("did you mean `@unique`?"));
}

#[test]
fn block_unique_creates_composite() {
    let src = r#"
table user {
  account string
  email   string

  @@unique(fields: [account, email])
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "user");
    assert_eq!(table.indexes.len(), 1);
    assert_eq!(table.indexes[0].fields, vec!["account", "email"]);
    assert!(matches!(table.indexes[0].kind, IndexKind::Unique));
    assert_eq!(table.indexes[0].name, "user_account_email_unique");
}

#[test]
fn block_index_with_explicit_name() {
    let src = r#"
table user {
  account string
  email   string

  @@index(fields: [account, email], name: "lookup_idx")
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "user");
    assert_eq!(table.indexes[0].name, "lookup_idx");
}

#[test]
fn block_index_unknown_field_errors() {
    let src = r#"
table user {
  email string

  @@index(fields: [nonexistent])
}
"#;
    let err = parse_validated(src).unwrap_err();
    let AurelineError::Validation(errs) = err else {
        panic!("expected validation error");
    };
    assert!(
        errs[0].message.contains("nonexistent"),
        "{}",
        errs[0].message
    );
}

#[test]
fn block_count_creates_count_index() {
    let src = r#"
table user {
  email string

  @@count
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "user");
    let count_idx = table
        .indexes
        .iter()
        .find(|i| matches!(i.kind, IndexKind::Count))
        .unwrap();
    assert!(count_idx.fields.is_empty());
    assert_eq!(count_idx.name, "user_count");
}

#[test]
fn analyzer_round_trips_through_parse_validated() {
    let src = r#"
analyzer edu {
  tokenizers blank, class
  filters lowercase, snowball(english)
}
"#;
    let schema = parse_validated(src).unwrap();
    let analyzer = schema
        .items
        .iter()
        .find_map(|i| match i {
            SchemaItem::AnalyzerDecl(a) => Some(a.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(analyzer.name, "edu");
    assert_eq!(analyzer.tokenizers, vec!["blank", "class"]);
    assert_eq!(analyzer.filters.len(), 2);
    assert_eq!(analyzer.filters[0].name, "lowercase");
    assert!(analyzer.filters[0].args.is_empty());
    assert_eq!(analyzer.filters[1].name, "snowball");
    assert_eq!(analyzer.filters[1].args, vec!["english"]);
}

#[test]
fn field_unique_with_name_keyword() {
    let src = r#"
table user {
  email string @unique(name: "user_email_lookup")
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "user");
    assert_eq!(table.indexes[0].name, "user_email_lookup");
    assert!(matches!(table.indexes[0].kind, IndexKind::Unique));
}

#[test]
fn field_index_with_name_keyword() {
    let src = r#"
table user {
  status string @index(name: "idx_user_status")
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "user");
    assert_eq!(table.indexes[0].name, "idx_user_status");
}

#[test]
fn field_hnsw_with_name_keyword() {
    let src = r#"
table doc {
  embedding array<float> @hnsw(dimension: 768, dist: euclidean, name: "emb_v3")
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "doc");
    assert_eq!(table.indexes[0].name, "emb_v3");
    match &table.indexes[0].kind {
        IndexKind::Hnsw {
            dimension, dist, ..
        } => {
            assert_eq!(*dimension, 768);
            assert_eq!(dist.as_deref(), Some("euclidean"));
        }
        _ => panic!("expected Hnsw"),
    }
}

#[test]
fn field_fulltext_with_name_keyword() {
    let src = r#"
table article {
  body string @fulltext(analyzer: simple, highlights: true, name: "body_search")
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "article");
    assert_eq!(table.indexes[0].name, "body_search");
    match &table.indexes[0].kind {
        IndexKind::Fulltext {
            analyzer,
            highlights,
            ..
        } => {
            assert_eq!(analyzer, "simple");
            assert!(*highlights);
        }
        _ => panic!("expected Fulltext"),
    }
}

#[test]
fn explicit_names_dont_collide_with_auto_names() {
    let src = r#"
table user {
  email  string @unique(name: "custom_email_name")
  status string @index
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "user");
    let names: Vec<&str> = table.indexes.iter().map(|i| i.name.as_str()).collect();
    assert!(names.contains(&"custom_email_name"));
    assert!(names.contains(&"user_status_idx"));
}

#[test]
fn multiple_field_annotations_all_apply() {
    let src = r#"
table doc {
  meta object @flexible
  embedding array<float> @hnsw(dimension: 1536)
  body string @fulltext(analyzer: simple)
}
"#;
    let schema = parse_validated(src).unwrap();
    let table = extract_table(&schema, "doc");
    assert!(table.fields[0].flexible);
    assert_eq!(table.indexes.len(), 2); // hnsw + fulltext
}
