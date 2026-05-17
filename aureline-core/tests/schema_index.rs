use aureline_core::ast::{IndexKind, Schema, SchemaItem};
use aureline_core::parse_validated;
use aureline_core::schema_index::SchemaIndex;
use aureline_test_support::aureline_schema;

fn parse_schema(source: &str) -> Schema {
    parse_validated(source).expect("schema should be valid")
}

fn tiny_schema() -> Schema {
    parse_schema(aureline_schema!(
        "analyzer edu_analyzer {",
        "  tokenizers blank, class",
        "  filters    lowercase, snowball(english)",
        "}",
        "",
        "table user schemafull {",
        "  email     string @unique",
        "  username  string @unique(name: \"user_username_idx\")",
        "  status    string @index",
        "  created   datetime",
        "  metadata  object @flexible",
        "}",
    ))
}

#[test]
fn schema_index_builds() {
    let schema = tiny_schema();
    let index = SchemaIndex::from_schema(&schema);

    assert!(index.tables.contains_key("user"));
    assert_eq!(index.tables.len(), 1);

    assert!(index.analyzers.contains_key("edu_analyzer"));
    assert_eq!(index.analyzers.len(), 1);

    assert!(index.get_field("user", "email").is_some());
    assert!(index.get_field("user", "username").is_some());
    assert!(index.get_field("user", "status").is_some());
    assert!(index.get_field("user", "created").is_some());
    assert!(index.get_field("user", "metadata").is_some());
    assert!(index.get_field("nonexistent", "field").is_none());
    assert!(index.get_field("user", "nonexistent").is_none());
    assert_eq!(index.fields.len(), 5);

    assert!(index.get_index("user", "user_email_unique").is_some());
    assert!(index.get_index("user", "user_username_idx").is_some());
    assert!(index.get_index("user", "user_status_idx").is_some());
    assert!(index.get_index("nonexistent", "idx").is_none());
    assert_eq!(index.indexes.len(), 3);
}

#[test]
fn schema_index_fields_deterministic_order() {
    let schema = tiny_schema();
    let index = SchemaIndex::from_schema(&schema);

    // BTreeMap iteration order is sorted, which for fields means
    // alphabetical by (table_name, field_name).
    let keys: Vec<_> = index
        .fields()
        .map(|(k, _)| (k.as_tuple().0, k.as_tuple().1))
        .collect();
    assert_eq!(
        keys,
        vec![
            ("user", "created"),
            ("user", "email"),
            ("user", "metadata"),
            ("user", "status"),
            ("user", "username"),
        ]
    );
}

fn full_schema() -> Schema {
    parse_schema(aureline_schema!(
        "analyzer edu_analyzer {",
        "  tokenizers blank, class",
        "  filters    lowercase, snowball(english)",
        "}",
        "",
        "table user schemafull {",
        "  email     string @unique",
        "  username  string @unique(name: \"user_username_idx\")",
        "  status    string @index",
        "  created   datetime",
        "  metadata  object @flexible",
        "}",
        "",
        "table lesson_chunk schemafull {",
        "  text       string @fulltext(analyzer: edu_analyzer, bm25: (1.2, 0.75))",
        "  embedding  array<float> @hnsw(dimension: 1536, dist: cosine, type: f32)",
        "  metadata   object @flexible",
        "}",
    ))
}

#[test]
fn schema_index_has_methods() {
    let schema = full_schema();
    let index = SchemaIndex::from_schema(&schema);

    assert!(index.has_table("user"));
    assert!(index.has_table("lesson_chunk"));
    assert!(!index.has_table("nonexistent"));

    assert!(index.has_analyzer("edu_analyzer"));
    assert!(!index.has_analyzer("nonexistent"));

    assert!(index.has_field("user", "email"));
    assert!(index.has_field("lesson_chunk", "text"));
    assert!(!index.has_field("user", "nonexistent"));
    assert!(!index.has_field("nonexistent", "field"));

    assert!(index.has_index("user", "user_email_unique"));
    assert!(index.has_index("user", "user_username_idx"));
    assert!(index.has_index("user", "user_status_idx"));
    assert!(!index.has_index("user", "nonexistent"));
    assert!(!index.has_index("nonexistent", "idx"));
}

#[test]
fn schema_index_iteration_deterministic() {
    let schema = full_schema();
    let index = SchemaIndex::from_schema(&schema);

    let tables_a: Vec<_> = index.tables().collect();
    let tables_b: Vec<_> = index.tables().collect();
    assert_eq!(tables_a, tables_b);

    let analyzers_a: Vec<_> = index.analyzers().collect();
    let analyzers_b: Vec<_> = index.analyzers().collect();
    assert_eq!(analyzers_a, analyzers_b);

    let fields_a: Vec<_> = index.fields().collect();
    let fields_b: Vec<_> = index.fields().collect();
    assert_eq!(fields_a, fields_b);

    let indexes_a: Vec<_> = index.indexes().collect();
    let indexes_b: Vec<_> = index.indexes().collect();
    assert_eq!(indexes_a, indexes_b);
}

#[test]
fn schema_index_table_scoped_iteration() {
    let schema = full_schema();
    let index = SchemaIndex::from_schema(&schema);

    let user_fields: Vec<_> = index.fields_for_table("user").collect();
    assert_eq!(user_fields.len(), 5);
    assert!(user_fields.iter().all(|(n, _)| !n.is_empty()));

    let chunk_fields: Vec<_> = index.fields_for_table("lesson_chunk").collect();
    assert_eq!(chunk_fields.len(), 3);

    let nonexistent: Vec<_> = index.fields_for_table("nonexistent").collect();
    assert!(nonexistent.is_empty());

    let user_indexes: Vec<_> = index.indexes_for_table("user").collect();
    assert_eq!(user_indexes.len(), 3);

    let chunk_indexes: Vec<_> = index.indexes_for_table("lesson_chunk").collect();
    assert_eq!(chunk_indexes.len(), 2);

    let nonexistent_idx: Vec<_> = index.indexes_for_table("nonexistent").collect();
    assert!(nonexistent_idx.is_empty());
}

#[test]
fn schema_index_fulltext_indexes() {
    let schema = full_schema();
    let index = SchemaIndex::from_schema(&schema);

    let fulltext: Vec<_> = index.fulltext_indexes().collect();
    assert_eq!(fulltext.len(), 1);
    assert_eq!(fulltext[0].0, "lesson_chunk");
    assert!(matches!(fulltext[0].1.kind, IndexKind::Fulltext { .. }));

    let using_analyzer: Vec<_> = index.indexes_using_analyzer("edu_analyzer").collect();
    assert_eq!(using_analyzer.len(), 1);
    assert_eq!(using_analyzer[0].0, "lesson_chunk");

    let wrong_analyzer: Vec<_> = index.indexes_using_analyzer("nonexistent").collect();
    assert!(wrong_analyzer.is_empty());
}

#[test]
fn schema_index_hnsw_indexes() {
    let schema = full_schema();
    let index = SchemaIndex::from_schema(&schema);

    let hnsw: Vec<_> = index.hnsw_indexes().collect();
    assert_eq!(hnsw.len(), 1);
    assert_eq!(hnsw[0].0, "lesson_chunk");
    assert!(matches!(hnsw[0].1.kind, IndexKind::Hnsw { .. }));
}

#[test]
fn schema_index_indexes_for_field() {
    let schema = full_schema();
    let index = SchemaIndex::from_schema(&schema);

    let email_indexes: Vec<_> = index.indexes_for_field("user", "email").collect();
    assert_eq!(email_indexes.len(), 1);
    assert_eq!(email_indexes[0].name, "user_email_unique");

    let status_indexes: Vec<_> = index.indexes_for_field("user", "status").collect();
    assert_eq!(status_indexes.len(), 1);
    assert_eq!(status_indexes[0].name, "user_status_idx");

    let username_indexes: Vec<_> = index.indexes_for_field("user", "username").collect();
    assert_eq!(username_indexes.len(), 1);
    assert_eq!(username_indexes[0].name, "user_username_idx");

    let nonexistent: Vec<_> = index.indexes_for_field("nonexistent", "field").collect();
    assert!(nonexistent.is_empty());

    let text_indexes: Vec<_> = index.indexes_for_field("lesson_chunk", "text").collect();
    assert_eq!(text_indexes.len(), 1);
}

#[test]
fn schema_index_doc_comments_ignored() {
    let schema = parse_schema(aureline_schema!(
        "/// This is a doc comment that should be ignored",
        "table user schemafull {",
        "  email string @unique",
        "}",
    ));
    let index = SchemaIndex::from_schema(&schema);
    assert_eq!(index.tables.len(), 1);
    assert!(index.has_table("user"));
}

#[test]
fn schema_index_parity_with_manual_traversal() {
    let schema = full_schema();
    let index = SchemaIndex::from_schema(&schema);

    let mut manual_analyzers = 0usize;
    let mut manual_tables = 0usize;
    let mut manual_fields = 0usize;
    let mut manual_indexes = 0usize;

    for item in &schema.items {
        match item {
            SchemaItem::AnalyzerDecl(_) => manual_analyzers += 1,
            SchemaItem::TableDecl(table) => {
                manual_tables += 1;
                manual_fields += table.fields.len();
                manual_indexes += table.indexes.len();
            }
            SchemaItem::DocComment { .. } | SchemaItem::FunctionDecl(_) => {}
        }
    }

    assert_eq!(index.analyzers.len(), manual_analyzers);
    assert_eq!(index.tables.len(), manual_tables);
    assert_eq!(index.fields.len(), manual_fields);
    assert_eq!(index.indexes.len(), manual_indexes);
}
