use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::journal::JournalEntry;

use super::client::{Client, ClientError};

/// One row in the `_aureline_migrations` tracking table on the target DB.
#[derive(Debug, Clone, Deserialize)]
pub struct AppliedRecord {
    pub idx: u32,
    pub name: String,
    pub checksum: String,
    pub applied_at: DateTime<Utc>,
}

/// Idempotently bootstrap the connection: create the namespace and database
/// if missing, then the `_aureline_migrations` tracking table inside them.
/// Runs as a single multi-statement query so the `USE NS .. DB ..` switch
/// happens before the table definitions, then switches the SDK connection's
/// scope so subsequent calls don't have to re-state it.
pub async fn ensure_tracking_table(client: &Client) -> Result<(), ClientError> {
    let conn = client.connection();
    let sql = format!(
        "\
DEFINE NAMESPACE IF NOT EXISTS {ns};
USE NS {ns};
DEFINE DATABASE IF NOT EXISTS {db};
USE NS {ns} DB {db};
DEFINE TABLE IF NOT EXISTS _aureline_migrations SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS idx        ON _aureline_migrations TYPE int;
DEFINE FIELD IF NOT EXISTS name       ON _aureline_migrations TYPE string;
DEFINE FIELD IF NOT EXISTS checksum   ON _aureline_migrations TYPE string;
DEFINE FIELD IF NOT EXISTS applied_at ON _aureline_migrations TYPE datetime;
",
        ns = conn.namespace,
        db = conn.database,
    );
    client.run_sql(&sql).await?;
    client.use_ns_db().await?;
    Ok(())
}

/// Read every row from `_aureline_migrations`. Returns an empty vec if the
/// table exists but is empty.
pub async fn read_applied(client: &Client) -> Result<Vec<AppliedRecord>, ClientError> {
    let mut response = client
        .db()
        .query("SELECT idx, name, checksum, applied_at FROM _aureline_migrations;")
        .await?;
    let rows: Vec<AppliedRecord> = response
        .take(0)
        .map_err(|e| ClientError::Decode(format!("decoding _aureline_migrations: {e}")))?;
    Ok(rows)
}

/// Insert a row into `_aureline_migrations` after a migration runs successfully.
/// The record id is `_aureline_migrations:<idx>` so re-applying the same idx
/// would conflict (which is the right behavior — caller checks `read_applied`
/// first).
pub async fn record_applied(client: &Client, entry: &JournalEntry) -> Result<(), ClientError> {
    let now = Utc::now();
    let sql = "CREATE type::thing('_aureline_migrations', $idx) CONTENT {
        idx: $idx,
        name: $name,
        checksum: $checksum,
        applied_at: $applied_at
    };";
    client
        .db()
        .query(sql)
        .bind(("idx", entry.idx))
        .bind(("name", entry.name.clone()))
        .bind(("checksum", entry.checksum.clone()))
        .bind(("applied_at", surrealdb::sql::Datetime::from(now)))
        .await?
        .check()?;
    Ok(())
}
