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
                SchemaItem::DocComment { .. } | SchemaItem::FunctionDecl(_) => {}
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
    pub fn get_analyzer(&self, name: &str) -> Option<&'a Analyzer> {
        self.analyzers.get(name).copied()
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
    pub fn get_table(&self, name: &str) -> Option<&'a Table> {
        self.tables.get(name).copied()
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
