use aureline_core::emit::emit_schema;
use aureline_test_support::{aureline_schema, expected_surql, parse_schema};

use super::assert_validation_contains;

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
fn hnsw_is_field_level_only() {
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
