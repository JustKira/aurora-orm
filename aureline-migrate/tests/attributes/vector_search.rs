use super::common::{diff_down, diff_up, empty_schema, parse_schema};

// SurrealDB accepts more HNSW tuning knobs than Aureline can currently model:
// M0, LM, EXTEND_CANDIDATES, KEEP_PRUNED_CONNECTIONS, HASHED_VECTOR, and the
// MINKOWSKI distance parameter. Those are schema/parser gaps, so this file only
// covers the HNSW surface the current Aureline AST can express.

#[test]
fn adds_hnsw_index_with_required_dimension() {
    let prev = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float>",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float> @hnsw(dimension: 1536)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "DEFINE INDEX document_embedding_hnsw ON document FIELDS embedding HNSW DIMENSION 1536;"
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX document_embedding_hnsw ON TABLE document;")
    );
}

#[test]
fn adds_hnsw_index_with_distance() {
    let prev = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float>",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float> @hnsw(dimension: 1536, dist: cosine)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "DEFINE INDEX document_embedding_hnsw ON document FIELDS embedding HNSW DIMENSION 1536 DIST COSINE;"
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX document_embedding_hnsw ON TABLE document;")
    );
}

#[test]
fn adds_hnsw_indexes_for_supported_distance_tokens() {
    let prev = parse_schema(aureline_schema!(
        "table Document {",
        "  chebyshev array<float>",
        "  cosine array<float>",
        "  euclidean array<float>",
        "  hamming array<float>",
        "  jaccard array<float>",
        "  manhattan array<float>",
        "  pearson array<float>",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
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
        diff_up(&prev, &next),
        expected_surql!(
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
fn adds_hnsw_index_with_type() {
    let prev = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float>",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float> @hnsw(dimension: 1536, type: f32)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "DEFINE INDEX document_embedding_hnsw ON document FIELDS embedding HNSW DIMENSION 1536 TYPE F32;"
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX document_embedding_hnsw ON TABLE document;")
    );
}

#[test]
fn adds_hnsw_indexes_for_supported_vector_types() {
    let prev = parse_schema(aureline_schema!(
        "table Document {",
        "  f64_embedding array<float>",
        "  f32_embedding array<float>",
        "  i64_embedding array<float>",
        "  i32_embedding array<float>",
        "  i16_embedding array<float>",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table Document {",
        "  f64_embedding array<float> @hnsw(dimension: 4, type: f64)",
        "  f32_embedding array<float> @hnsw(dimension: 4, type: f32)",
        "  i64_embedding array<float> @hnsw(dimension: 4, type: i64)",
        "  i32_embedding array<float> @hnsw(dimension: 4, type: i32)",
        "  i16_embedding array<float> @hnsw(dimension: 4, type: i16)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "DEFINE INDEX document_f64_embedding_hnsw ON document FIELDS f64_embedding HNSW DIMENSION 4 TYPE F64;",
            "DEFINE INDEX document_f32_embedding_hnsw ON document FIELDS f32_embedding HNSW DIMENSION 4 TYPE F32;",
            "DEFINE INDEX document_i64_embedding_hnsw ON document FIELDS i64_embedding HNSW DIMENSION 4 TYPE I64;",
            "DEFINE INDEX document_i32_embedding_hnsw ON document FIELDS i32_embedding HNSW DIMENSION 4 TYPE I32;",
            "DEFINE INDEX document_i16_embedding_hnsw ON document FIELDS i16_embedding HNSW DIMENSION 4 TYPE I16;",
        )
    );
}

#[test]
fn adds_hnsw_index_with_efc_and_m() {
    let prev = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float>",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float> @hnsw(dimension: 1536, efc: 200, m: 16)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "DEFINE INDEX document_embedding_hnsw ON document FIELDS embedding HNSW DIMENSION 1536 EFC 200 M 16;"
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX document_embedding_hnsw ON TABLE document;")
    );
}

#[test]
fn canonicalizes_hnsw_option_order() {
    let prev = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float>",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float> @hnsw(m: 16, efc: 200, dist: cosine, type: f32, dimension: 1536)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "DEFINE INDEX document_embedding_hnsw ON document FIELDS embedding HNSW DIMENSION 1536 TYPE F32 DIST COSINE EFC 200 M 16;"
        )
    );
}

#[test]
fn adds_hnsw_index_with_all_current_aureline_options() {
    let prev = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float>",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float> @hnsw(dimension: 1536, type: f32, dist: cosine, efc: 200, m: 16)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "DEFINE INDEX document_embedding_hnsw ON document FIELDS embedding HNSW DIMENSION 1536 TYPE F32 DIST COSINE EFC 200 M 16;"
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX document_embedding_hnsw ON TABLE document;")
    );
}

#[test]
fn creates_table_with_hnsw_index_after_fields() {
    let next = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float> @hnsw(dimension: 1536, dist: cosine)",
        "}",
    ));

    assert_eq!(
        diff_up(&empty_schema(), &next),
        expected_surql!(
            "DEFINE TABLE document;",
            "DEFINE FIELD embedding ON document TYPE array<float>;",
            "DEFINE INDEX document_embedding_hnsw ON document FIELDS embedding HNSW DIMENSION 1536 DIST COSINE;",
        )
    );
}

#[test]
fn removes_hnsw_index_from_existing_table() {
    let prev = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float> @hnsw(dimension: 1536, dist: cosine)",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float>",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("REMOVE INDEX document_embedding_hnsw ON TABLE document;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "DEFINE INDEX document_embedding_hnsw ON document FIELDS embedding HNSW DIMENSION 1536 DIST COSINE;"
        )
    );
}

#[test]
fn changes_hnsw_options_by_replacing_the_index() {
    let prev = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float> @hnsw(dimension: 768, dist: euclidean, name: embedding_ann)",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table Document {",
        "  embedding array<float> @hnsw(dimension: 1536, dist: cosine, type: f32, efc: 200, m: 16, name: embedding_ann)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "REMOVE INDEX embedding_ann ON TABLE document;",
            "DEFINE INDEX embedding_ann ON document FIELDS embedding HNSW DIMENSION 1536 TYPE F32 DIST COSINE EFC 200 M 16;",
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "REMOVE INDEX embedding_ann ON TABLE document;",
            "DEFINE INDEX embedding_ann ON document FIELDS embedding HNSW DIMENSION 768 DIST EUCLIDEAN;",
        )
    );
}
