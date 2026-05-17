use std::fs;
use std::path::Path;

use aureline_core::ast::{Schema, SchemaItem};

use crate::error::{Error, Result, io};
use crate::journal::Journal;
use crate::snapshot::parse_snapshot;

/// Writes to a sibling `*.tmp` then renames over `path`, so a crash mid-write
/// can never leave a half-written file at `path`.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| io(parent, error))?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes).map_err(|error| io(&tmp, error))?;
    fs::rename(&tmp, path).map_err(|error| io(path, error))
}

pub fn read_schema(path: &Path) -> Result<Schema> {
    let source = fs::read_to_string(path).map_err(|error| io(path, error))?;
    let schema = aureline_core::parse_validated(&source).map_err(|source| Error::Parse {
        path: path.display().to_string(),
        source,
    })?;
    reject_unsupported_schema_items(path, &schema)?;
    Ok(schema)
}

fn reject_unsupported_schema_items(path: &Path, schema: &Schema) -> Result<()> {
    for item in &schema.items {
        if let SchemaItem::FunctionDecl(function) = item {
            return Err(Error::UnsupportedSchemaItem {
                path: path.display().to_string(),
                message: format!(
                    "function '{}' is not supported by aureline-migrate yet and would not be migrated or snapshotted",
                    function.name
                ),
            });
        }
    }
    Ok(())
}

pub fn read_previous_schema(meta_dir: &Path, journal: &Journal) -> Result<Schema> {
    let Some(last) = journal.entries.iter().max_by_key(|entry| entry.idx) else {
        return Ok(Schema { items: Vec::new() });
    };
    let path = meta_dir.join(format!("{:04}_snapshot.json", last.idx));
    let contents = fs::read_to_string(&path).map_err(|error| io(&path, error))?;
    parse_snapshot(&contents)
}
