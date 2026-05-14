use aureline_core::emit::emit_schema;
use aureline_test_support::{aureline_schema, expected_surql, parse_schema, validation_errors};

// SurrealDB DEFINE INDEX also has statement-level clauses like OVERWRITE,
// IF NOT EXISTS, COMMENT, CONCURRENTLY, DEFER, and the COLUMNS alias. Aureline
// does not model those in schema attributes yet, so these tests focus on the
// index definitions Aureline can currently express and emit.

fn assert_validation_contains(source: &str, expected: &str) {
    let errors = validation_errors(source);
    assert!(
        errors.iter().any(|error| error.message.contains(expected)),
        "expected validation error containing `{expected}`, got {errors:#?}"
    );
}

#[test]
fn emits_standard_unique_composite_and_count_indexes() {
    let schema = parse_schema(aureline_schema!(
        "table User {",
        "  tenant string",
        "  email string @index",
        "  handle string @unique",
        "",
        "  @@index(fields: [tenant, email], name: tenant_email_lookup)",
        "  @@unique(fields: [tenant, handle])",
        "  @@count",
        "}",
    ));

    assert_eq!(
        emit_schema(&schema),
        expected_surql!(
            "DEFINE TABLE user;",
            "DEFINE FIELD email ON user TYPE string;",
            "DEFINE FIELD handle ON user TYPE string;",
            "DEFINE FIELD tenant ON user TYPE string;",
            "DEFINE INDEX tenant_email_lookup ON user FIELDS tenant, email;",
            "DEFINE INDEX user_count ON user COUNT;",
            "DEFINE INDEX user_email_idx ON user FIELDS email;",
            "DEFINE INDEX user_handle_unique ON user FIELDS handle UNIQUE;",
            "DEFINE INDEX user_tenant_handle_unique ON user FIELDS tenant, handle UNIQUE;",
        )
    );
}

#[test]
fn emits_fulltext_index_with_explicit_analyzer_options() {
    let schema = parse_schema(aureline_schema!(
        "analyzer simple {",
        "  tokenizers blank, class",
        "  filters lowercase",
        "}",
        "",
        "table Article {",
        "  body string @fulltext(analyzer: simple, bm25: (1.2, 0.75), highlights: true)",
        "}",
    ));

    assert_eq!(
        emit_schema(&schema),
        expected_surql!(
            "DEFINE ANALYZER simple TOKENIZERS blank,class FILTERS lowercase;",
            "DEFINE TABLE article;",
            "DEFINE FIELD body ON article TYPE string;",
            "DEFINE INDEX article_body_fts ON article FIELDS body FULLTEXT ANALYZER simple BM25(1.2, 0.75) HIGHLIGHTS;",
        )
    );
}

#[test]
fn emits_hnsw_index_with_current_aureline_options() {
    let schema = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float> @hnsw(m: 16, efc: 200, dist: cosine, type: f32, dimension: 1536)",
        "}",
    ));

    assert_eq!(
        emit_schema(&schema),
        expected_surql!(
            "DEFINE TABLE document;",
            "DEFINE FIELD embedding ON document TYPE array<float>;",
            "DEFINE INDEX document_embedding_hnsw ON document FIELDS embedding HNSW DIMENSION 1536 TYPE F32 DIST COSINE EFC 200 M 16;",
        )
    );
}

#[test]
fn accepts_supported_hnsw_distance_tokens() {
    let schema = parse_schema(aureline_schema!(
        "table Document {",
        "  chebyshev array<float> @hnsw(dimension: 4, dist: chebyshev)",
        "  cosine array<float> @hnsw(dimension: 4, dist: cosine)",
        "  euclidean array<float> @hnsw(dimension: 4, dist: euclidean)",
        "  hamming array<float> @hnsw(dimension: 4, dist: hamming)",
        "  jaccard array<float> @hnsw(dimension: 4, dist: jaccard)",
        "  manhattan array<float> @hnsw(dimension: 4, dist: manhattan)",
        "  pearson array<float> @hnsw(dimension: 4, dist: pearson)",
        "}",
    ));

    assert_eq!(
        emit_schema(&schema),
        expected_surql!(
            "DEFINE TABLE document;",
            "DEFINE FIELD chebyshev ON document TYPE array<float>;",
            "DEFINE FIELD cosine ON document TYPE array<float>;",
            "DEFINE FIELD euclidean ON document TYPE array<float>;",
            "DEFINE FIELD hamming ON document TYPE array<float>;",
            "DEFINE FIELD jaccard ON document TYPE array<float>;",
            "DEFINE FIELD manhattan ON document TYPE array<float>;",
            "DEFINE FIELD pearson ON document TYPE array<float>;",
            "DEFINE INDEX document_chebyshev_hnsw ON document FIELDS chebyshev HNSW DIMENSION 4 DIST CHEBYSHEV;",
            "DEFINE INDEX document_cosine_hnsw ON document FIELDS cosine HNSW DIMENSION 4 DIST COSINE;",
            "DEFINE INDEX document_euclidean_hnsw ON document FIELDS euclidean HNSW DIMENSION 4 DIST EUCLIDEAN;",
            "DEFINE INDEX document_hamming_hnsw ON document FIELDS hamming HNSW DIMENSION 4 DIST HAMMING;",
            "DEFINE INDEX document_jaccard_hnsw ON document FIELDS jaccard HNSW DIMENSION 4 DIST JACCARD;",
            "DEFINE INDEX document_manhattan_hnsw ON document FIELDS manhattan HNSW DIMENSION 4 DIST MANHATTAN;",
            "DEFINE INDEX document_pearson_hnsw ON document FIELDS pearson HNSW DIMENSION 4 DIST PEARSON;",
        )
    );
}

#[test]
fn accepts_supported_hnsw_vector_types() {
    let schema = parse_schema(aureline_schema!(
        "table Document {",
        "  f64_embedding array<float> @hnsw(dimension: 4, type: f64)",
        "  f32_embedding array<float> @hnsw(dimension: 4, type: f32)",
        "  i64_embedding array<float> @hnsw(dimension: 4, type: i64)",
        "  i32_embedding array<float> @hnsw(dimension: 4, type: i32)",
        "  i16_embedding array<float> @hnsw(dimension: 4, type: i16)",
        "}",
    ));

    assert_eq!(
        emit_schema(&schema),
        expected_surql!(
            "DEFINE TABLE document;",
            "DEFINE FIELD f32_embedding ON document TYPE array<float>;",
            "DEFINE FIELD f64_embedding ON document TYPE array<float>;",
            "DEFINE FIELD i16_embedding ON document TYPE array<float>;",
            "DEFINE FIELD i32_embedding ON document TYPE array<float>;",
            "DEFINE FIELD i64_embedding ON document TYPE array<float>;",
            "DEFINE INDEX document_f32_embedding_hnsw ON document FIELDS f32_embedding HNSW DIMENSION 4 TYPE F32;",
            "DEFINE INDEX document_f64_embedding_hnsw ON document FIELDS f64_embedding HNSW DIMENSION 4 TYPE F64;",
            "DEFINE INDEX document_i16_embedding_hnsw ON document FIELDS i16_embedding HNSW DIMENSION 4 TYPE I16;",
            "DEFINE INDEX document_i32_embedding_hnsw ON document FIELDS i32_embedding HNSW DIMENSION 4 TYPE I32;",
            "DEFINE INDEX document_i64_embedding_hnsw ON document FIELDS i64_embedding HNSW DIMENSION 4 TYPE I64;",
        )
    );
}

#[test]
fn composite_index_fields_must_exist_be_non_empty_and_unique() {
    assert_validation_contains(
        aureline_schema!(
            "table User {",
            "  email string",
            "",
            "  @@index(fields: [])",
            "}",
        ),
        "at least one field",
    );
    assert_validation_contains(
        aureline_schema!(
            "table User {",
            "  email string",
            "",
            "  @@unique(fields: [email, email])",
            "}",
        ),
        "duplicate field `email`",
    );
    assert_validation_contains(
        aureline_schema!(
            "table User {",
            "  email string",
            "",
            "  @@index(fields: [missing])",
            "}",
        ),
        "unknown field `missing`",
    );
}

#[test]
fn count_index_is_table_level_only() {
    assert_validation_contains(
        aureline_schema!(
            "table User {",
            "  email string",
            "",
            "  @@count(fields: [email])",
            "}",
        ),
        "@@count on User takes no arguments",
    );
}

#[test]
fn fulltext_and_hnsw_are_field_level_only() {
    assert_validation_contains(
        aureline_schema!(
            "table Article {",
            "  body string",
            "",
            "  @@fulltext(fields: [body])",
            "}",
        ),
        "unknown block attribute `@@fulltext`",
    );
    assert_validation_contains(
        aureline_schema!(
            "table Document {",
            "  embedding array<float>",
            "",
            "  @@hnsw(fields: [embedding])",
            "}",
        ),
        "unknown block attribute `@@hnsw`",
    );
}

#[test]
fn fulltext_requires_string_and_explicit_analyzer() {
    assert_validation_contains(
        aureline_schema!("table Article {", "  body string @fulltext", "}",),
        "analyzer",
    );
    assert_validation_contains(
        aureline_schema!(
            "table Article {",
            "  body bytes @fulltext(analyzer: simple)",
            "}",
        ),
        "requires `string`",
    );
}

#[test]
fn hnsw_rejects_unsupported_or_out_of_range_options() {
    assert_validation_contains(
        aureline_schema!(
            "table Document {",
            "  embedding array<float> @hnsw(dimension: 4, dist: bogus)",
            "}",
        ),
        "unknown @hnsw dist `bogus`",
    );
    assert_validation_contains(
        aureline_schema!(
            "table Document {",
            "  embedding array<float> @hnsw(dimension: 4, dist: minkowski)",
            "}",
        ),
        "Aureline does not model yet",
    );
    assert_validation_contains(
        aureline_schema!(
            "table Document {",
            "  embedding array<float> @hnsw(dimension: 4, type: u8)",
            "}",
        ),
        "unknown @hnsw type `u8`",
    );
    assert_validation_contains(
        aureline_schema!(
            "table Document {",
            "  embedding array<float> @hnsw(dimension: 4, m: 128)",
            "}",
        ),
        "@hnsw m: expected a value <= 127",
    );
    assert_validation_contains(
        aureline_schema!(
            "table Document {",
            "  embedding array<float> @hnsw(dimension: 70000)",
            "}",
        ),
        "@hnsw dimension: expected a value <= 65535",
    );
}

#[test]
fn hnsw_rejects_surrealdb_options_not_modeled_by_aureline_yet() {
    assert_validation_contains(
        aureline_schema!(
            "table Document {",
            "  embedding array<float> @hnsw(dimension: 4, m0: 24)",
            "}",
        ),
        "unknown @hnsw arg `m0`",
    );
}

#[test]
fn duplicate_index_names_are_rejected_within_a_table() {
    assert_validation_contains(
        aureline_schema!(
            "table User {",
            "  email string @index(name: user_lookup)",
            "  handle string @unique(name: user_lookup)",
            "}",
        ),
        "duplicate index name `user_lookup` on table User",
    );
}
