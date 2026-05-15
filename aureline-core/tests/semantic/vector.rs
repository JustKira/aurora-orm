// Vector index tests cover the HNSW subset that Aureline currently models.
// SurrealDB has additional options; tests below document the accepted subset
// and the gaps we reject until the schema model grows.

use aureline_core::ast::IndexKind;
use aureline_test_support::aureline_schema;

use super::common::{assert_semantic_error_contains, checked_schema, table};

#[test]
fn hnsw_with_current_aureline_options_lowers_to_index() {
    let schema = checked_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float> @hnsw(dimension: 1536, dist: cosine, type: f32, efc: 200, m: 16, name: emb_v3)",
        "}",
    ));
    let table = table(&schema, "Document");

    assert_eq!(table.indexes[0].name, "emb_v3");
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
        other => panic!("expected hnsw index, got {other:?}"),
    }
}

#[test]
fn hnsw_requires_array_float_and_dimension() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table Document {",
            "  text string @hnsw(dimension: 1536)",
            "}",
        ),
        "requires `array<float>`",
    );
    assert_semantic_error_contains(
        aureline_schema!(
            "table Document {",
            "  embedding array<float> @hnsw(dist: cosine)",
            "}",
        ),
        "@hnsw requires a `dimension:` argument",
    );
}

#[test]
fn hnsw_accepts_supported_distances() {
    // These are the distance tokens Aureline can currently render without
    // extra parameters. Minkowski is tested separately because SurrealDB needs
    // an additional numeric argument for it.
    let schema = checked_schema(aureline_schema!(
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
    let table = table(&schema, "Document");

    assert_eq!(table.indexes.len(), 7);
}

#[test]
fn hnsw_accepts_supported_vector_types() {
    let schema = checked_schema(aureline_schema!(
        "table Document {",
        "  f64_embedding array<float> @hnsw(dimension: 4, type: f64)",
        "  f32_embedding array<float> @hnsw(dimension: 4, type: f32)",
        "  i64_embedding array<float> @hnsw(dimension: 4, type: i64)",
        "  i32_embedding array<float> @hnsw(dimension: 4, type: i32)",
        "  i16_embedding array<float> @hnsw(dimension: 4, type: i16)",
        "}",
    ));
    let table = table(&schema, "Document");

    assert_eq!(table.indexes.len(), 5);
}

#[test]
fn hnsw_rejects_unsupported_or_out_of_range_options() {
    assert_semantic_error_contains(
        aureline_schema!(
            "table Document {",
            "  embedding array<float> @hnsw(dimension: 4, dist: bogus)",
            "}",
        ),
        "unknown @hnsw dist `bogus`",
    );
    assert_semantic_error_contains(
        aureline_schema!(
            "table Document {",
            "  embedding array<float> @hnsw(dimension: 4, dist: minkowski)",
            "}",
        ),
        "Aureline does not model yet",
    );
    assert_semantic_error_contains(
        aureline_schema!(
            "table Document {",
            "  embedding array<float> @hnsw(dimension: 4, type: u8)",
            "}",
        ),
        "unknown @hnsw type `u8`",
    );
    assert_semantic_error_contains(
        aureline_schema!(
            "table Document {",
            "  embedding array<float> @hnsw(dimension: 4, m: 128)",
            "}",
        ),
        "@hnsw m: expected a value <= 127",
    );
    assert_semantic_error_contains(
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
    // These options exist in SurrealDB, but Aureline has no AST fields for
    // them yet. Rejecting them is safer than silently dropping migration data.
    assert_semantic_error_contains(
        aureline_schema!(
            "table Document {",
            "  embedding array<float> @hnsw(dimension: 4, m0: 24)",
            "}",
        ),
        "unknown @hnsw arg `m0`",
    );
    assert_semantic_error_contains(
        aureline_schema!(
            "table Document {",
            "  embedding array<float> @hnsw(dimension: 4, extend_candidates: true)",
            "}",
        ),
        "unknown @hnsw arg `extend_candidates`",
    );
}

#[test]
fn hnsw_is_field_level_only() {
    assert_semantic_error_contains(
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
