use aurora_core::ast::Schema;

use crate::error::{Error, Result};

mod canonical;
mod v1;

pub use canonical::canonicalize;

pub const SNAPSHOT_VERSION: u32 = 1;

pub fn parse_snapshot(json_str: &str) -> Result<Schema> {
    let snapshot = serde_json::from_str::<v1::Snapshot>(json_str)
        .map_err(|error| Error::SnapshotDecode(error.to_string()))?;

    match snapshot.version {
        SNAPSHOT_VERSION => v1::snapshot_v1_to_schema(snapshot),
        version => Err(Error::SnapshotDecode(format!(
            "unsupported snapshot version {version}"
        ))),
    }
}
