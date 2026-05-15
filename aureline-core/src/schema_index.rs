//! Query accelerator over a validated [`Schema`] with borrowed references.

use std::collections::BTreeMap;

use crate::ast::{Analyzer, Field, Index, IndexKind, Schema, SchemaItem, Table};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldKey<'a>(&'a str, &'a str);

/// A lookup key for an index: `(table_name, index_name)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexKey<'a>(&'a str, &'a str);

impl<'a> FieldKey<'a> {
    /// Returns the table name and field name respectively.
    #[inline]
    pub fn as_tuple(&self) -> (&'a str, &'a str) {
        (self.0, self.1)
    }
}

impl<'a> IndexKey<'a> {
    /// Returns the table name and index name respectively.
    #[inline]
    pub fn as_tuple(&self) -> (&'a str, &'a str) {
        (self.0, self.1)
    }
}

/// Borrowed index over a [`Schema`] for fast name-based lookups.
///
/// All maps store **borrowed references** (`&'a str`, `&'a T`) into the original
/// `Schema`. The `SchemaIndex` cannot outlive the `Schema` it was built from.
///
/// # Maps
///
/// - `analyzers` — keyed by analyzer name
/// - `tables` — keyed by table name
/// - `fields` — keyed by `(table_name, field_name)`
/// - `indexes` — keyed by `(table_name, index_name)`
///
/// All maps use [`BTreeMap`] for deterministic iteration order.
///
/// # Future Evolution
///
/// The current `SchemaIndex` is **borrowed** — it cannot outlive the `Schema`
/// it was built from. Future work will introduce an **owned** variant
/// (`ResolvedSchema`) that uses integer IDs (`TableId`, `FieldId`, `IndexId`,
/// `AnalyzerId`) instead of string names. This enables:
///
/// - **Ownership**: a `ResolvedSchema` can be stored in a struct or returned
///   from a function without tying its lifetime to the source `Schema`.
/// - **Dependency graph**: topological ordering of tables based on foreign-key
///   references for safe application of multi-table migration steps.
/// - **Codegen IR derivation**: a clean separation between the runtime lookup
///   index (`SchemaIndex`) and the compile-time type derivation pipeline
///   (`ResolvedSchema` → client types via wasm plugin).
///
/// The `SchemaIndex` remains focused on **query acceleration** over an existing
/// validated `Schema`; it intentionally does **not** own data or provide
/// mutation methods.
#[derive(Debug)]
pub struct SchemaIndex<'a> {
    /// Analyzers defined in the schema, keyed by name.
    pub analyzers: BTreeMap<&'a str, &'a Analyzer>,
    /// Tables defined in the schema, keyed by name.
    pub tables: BTreeMap<&'a str, &'a Table>,
    /// Fields indexed by `(table_name, field_name)`.
    pub fields: BTreeMap<FieldKey<'a>, &'a Field>,
    /// Indexes indexed by `(table_name, index_name)`.
    pub indexes: BTreeMap<IndexKey<'a>, &'a Index>,
}

impl<'a> SchemaIndex<'a> {
    /// Build a `SchemaIndex` from a validated `Schema`.
    ///
    /// The returned index borrows from `schema` and shares its lifetime.
    /// All tables, fields, and indexes in the schema are indexed by name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use aureline_core::{parse_validated, schema_index::SchemaIndex};
    ///
    /// let schema = parse_validated(r#"
    /// table user schemafull {
    ///   email string @unique
    /// }
    /// "#).unwrap();
    /// let index = SchemaIndex::from_schema(&schema);
    /// assert!(index.has_table("user"));
    /// ```
    #[inline]
    pub fn from_schema(schema: &'a Schema) -> Self {
        let mut analyzers = BTreeMap::new();
        let mut tables = BTreeMap::new();
        let mut fields = BTreeMap::new();
        let mut indexes = BTreeMap::new();

        for item in &schema.items {
            match item {
                SchemaItem::AnalyzerDecl(analyzer) => {
                    analyzers.insert(analyzer.name.as_str(), analyzer);
                }
                SchemaItem::TableDecl(table) => {
                    let table_name = table.name.as_str();
                    tables.insert(table_name, table);

                    for field in &table.fields {
                        let key = FieldKey(table_name, field.name.as_str());
                        fields.insert(key, field);
                    }

                    for index in &table.indexes {
                        let key = IndexKey(table_name, index.name.as_str());
                        indexes.insert(key, index);
                    }
                }
                SchemaItem::DocComment { .. } => {}
            }
        }

        Self {
            analyzers,
            tables,
            fields,
            indexes,
        }
    }

    /// Look up an analyzer by name.
    #[inline]
    pub fn get_analyzer(&self, name: &str) -> Option<&&'a Analyzer> {
        self.analyzers.get(name)
    }

    /// Look up a table by name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use aureline_core::{parse_validated, schema_index::SchemaIndex};
    ///
    /// let schema = parse_validated(r#"
    /// table user schemafull {
    ///   email string @unique
    /// }
    /// "#).unwrap();
    /// let index = SchemaIndex::from_schema(&schema);
    ///
    /// let user = index.get_table("user");
    /// assert!(user.is_some());
    /// ```
    #[inline]
    pub fn get_table(&self, name: &str) -> Option<&&'a Table> {
        self.tables.get(name)
    }

    /// Look up a field by table name and field name.
    ///
    /// Returns `None` if the table does not exist or the field is not present.
    ///
    /// ```rust
    /// use aureline_core::{parse_validated, schema_index::SchemaIndex};
    ///
    /// let schema = parse_validated(r#"
    /// table user schemafull {
    ///   email string @unique
    ///   name string
    /// }
    /// "#).unwrap();
    /// let index = SchemaIndex::from_schema(&schema);
    ///
    /// let email = index.get_field("user", "email");
    /// assert!(email.is_some());
    /// ```
    #[inline]
    pub fn get_field(&self, table_name: &str, field_name: &str) -> Option<&'a Field> {
        let table = self.tables.get(table_name)?;
        table.fields.iter().find(|f| f.name.as_str() == field_name)
    }

    /// Look up an index by table name and index name.
    ///
    /// Returns `None` if the table does not exist or the index is not present.
    #[inline]
    pub fn get_index(&self, table_name: &str, index_name: &str) -> Option<&'a Index> {
        let table = self.tables.get(table_name)?;
        table.indexes.iter().find(|i| i.name.as_str() == index_name)
    }

    // ─── Existence checks ─────────────────────────────────────────────────────

    /// Returns `true` if the schema contains a table with the given name.
    #[inline]
    pub fn has_table(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    /// Returns `true` if the schema defines an analyzer with the given name.
    #[inline]
    pub fn has_analyzer(&self, name: &str) -> bool {
        self.analyzers.contains_key(name)
    }

    /// Returns `true` if the given table has a field with the given name.
    #[inline]
    pub fn has_field(&self, table_name: &str, field_name: &str) -> bool {
        self.get_field(table_name, field_name).is_some()
    }

    /// Returns `true` if the given table has an index with the given name.
    #[inline]
    pub fn has_index(&self, table_name: &str, index_name: &str) -> bool {
        self.get_index(table_name, index_name).is_some()
    }

    // ─── Iteration ─────────────────────────────────────────────────────────────

    /// Iterate over all tables in deterministic (alphabetical) order.
    #[inline]
    pub fn tables(&self) -> impl Iterator<Item = (&str, &Table)> {
        self.tables.iter().map(|(&name, &table)| (name, table))
    }

    /// Iterate over all analyzers in deterministic (alphabetical) order.
    #[inline]
    pub fn analyzers(&self) -> impl Iterator<Item = (&str, &Analyzer)> {
        self.analyzers
            .iter()
            .map(|(&name, &analyzer)| (name, analyzer))
    }

    /// Iterate over all fields in deterministic (table_name, field_name) order.
    #[inline]
    pub fn fields(&self) -> impl Iterator<Item = (FieldKey<'a>, &Field)> {
        self.fields.iter().map(|(&key, &field)| (key, field))
    }

    /// Iterate over all indexes in deterministic (table_name, index_name) order.
    #[inline]
    pub fn indexes(&self) -> impl Iterator<Item = (IndexKey<'a>, &Index)> {
        self.indexes.iter().map(|(&key, &index)| (key, index))
    }

    // ─── Table-scoped iteration ───────────────────────────────────────────────

    /// Iterate over all fields belonging to a specific table.
    ///
    /// Returns an empty iterator if the table does not exist.
    #[inline]
    pub fn fields_for_table(&self, table_name: &str) -> impl Iterator<Item = (&str, &Field)> {
        self.tables
            .get(table_name)
            .map(|t| t.fields.iter().map(|f| (f.name.as_str(), f)))
            .into_iter()
            .flatten()
    }

    /// Iterate over all indexes belonging to a specific table.
    ///
    /// Returns an empty iterator if the table does not exist.
    #[inline]
    pub fn indexes_for_table(&self, table_name: &str) -> impl Iterator<Item = (&str, &Index)> {
        self.tables
            .get(table_name)
            .map(|t| t.indexes.iter().map(|i| (i.name.as_str(), i)))
            .into_iter()
            .flatten()
    }

    // ─── Relationship helpers ─────────────────────────────────────────────────

    /// Returns all indexes that reference the given field.
    ///
    /// An index "references" a field if the field appears in `index.fields`.
    pub fn indexes_for_field(
        &self,
        table_name: &str,
        field_name: &str,
    ) -> impl Iterator<Item = &Index> {
        self.indexes_for_table(table_name)
            .filter(move |(_, idx)| idx.fields.iter().any(|f| f.as_str() == field_name))
            .map(|(_, idx)| idx)
    }

    /// Returns all indexes that use a specific analyzer.
    ///
    /// Yields `(table_name, index)` pairs for fulltext indexes whose
    /// `analyzer` name matches `analyzer_name`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use aureline_core::{parse_validated, schema_index::SchemaIndex};
    ///
    /// let schema = parse_validated(r#"
    /// table user schemafull {
    ///   email string @unique
    ///   status string @index
    /// }
    /// "#).unwrap();
    /// let index = SchemaIndex::from_schema(&schema);
    ///
    /// // Find all indexes on the user table
    /// let user_indexes: Vec<_> = index.indexes_for_table("user").collect();
    /// assert_eq!(user_indexes.len(), 2);
    /// ```
    pub fn indexes_using_analyzer(
        &self,
        analyzer_name: &str,
    ) -> impl Iterator<Item = (&str, &Index)> {
        self.indexes
            .iter()
            .filter(move |(_, idx)| {
                if let IndexKind::Fulltext { analyzer, .. } = &idx.kind {
                    analyzer.as_str() == analyzer_name
                } else {
                    false
                }
            })
            .map(|(&IndexKey(table_name, _), &idx)| (table_name, idx))
    }

    /// Returns all fulltext indexes in the schema.
    ///
    /// Yields `(table_name, index)` pairs.
    #[inline]
    pub fn fulltext_indexes(&self) -> impl Iterator<Item = (&str, &Index)> {
        self.indexes
            .iter()
            .filter(|(_, idx)| matches!(idx.kind, IndexKind::Fulltext { .. }))
            .map(|(&IndexKey(table_name, _), &idx)| (table_name, idx))
    }

    /// Returns all HNSW vector indexes in the schema.
    ///
    /// Yields `(table_name, index)` pairs.
    #[inline]
    pub fn hnsw_indexes(&self) -> impl Iterator<Item = (&str, &Index)> {
        self.indexes
            .iter()
            .filter(|(_, idx)| matches!(idx.kind, IndexKind::Hnsw { .. }))
            .map(|(&IndexKey(table_name, _), &idx)| (table_name, idx))
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{IndexKind, Schema, SchemaItem};
    use aureline_test_support::aureline_schema;

    use super::SchemaIndex;

    fn parse_schema(source: &str) -> Schema {
        crate::parse_validated(source).expect("schema should be valid")
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

        // Tables
        assert!(index.tables.contains_key("user"));
        assert_eq!(index.tables.len(), 1);

        // Analyzers
        assert!(index.analyzers.contains_key("edu_analyzer"));
        assert_eq!(index.analyzers.len(), 1);

        // Fields — user table has 5 fields
        assert!(index.get_field("user", "email").is_some());
        assert!(index.get_field("user", "username").is_some());
        assert!(index.get_field("user", "status").is_some());
        assert!(index.get_field("user", "created").is_some());
        assert!(index.get_field("user", "metadata").is_some());
        assert!(index.get_field("nonexistent", "field").is_none());
        assert!(index.get_field("user", "nonexistent").is_none());
        assert_eq!(index.fields.len(), 5);

        // Indexes (auto-named: @unique → user_<field>_unique, @index → user_<field>_idx)
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
        let keys: Vec<_> = index.fields.keys().collect();
        assert_eq!(
            keys,
            vec![
                &super::FieldKey("user", "created"),
                &super::FieldKey("user", "email"),
                &super::FieldKey("user", "metadata"),
                &super::FieldKey("user", "status"),
                &super::FieldKey("user", "username"),
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
        assert!(user_fields.iter().all(|(n, _)| *n != ""));

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
                SchemaItem::DocComment { .. } => {}
            }
        }

        assert_eq!(index.analyzers.len(), manual_analyzers);
        assert_eq!(index.tables.len(), manual_tables);
        assert_eq!(index.fields.len(), manual_fields);
        assert_eq!(index.indexes.len(), manual_indexes);
    }
}
