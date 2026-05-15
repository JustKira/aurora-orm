use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;

use super::env::Connection;

/// Thin wrapper around the official SurrealDB SDK.
///
/// Construction (`connect`) opens the underlying connection and signs in as
/// root, but does **not** select a namespace/database. Callers run the
/// bootstrap query first (which may include `DEFINE NAMESPACE` / `DEFINE
/// DATABASE`) and then call `use_ns_db` to switch the connection's scope.
pub struct Client {
    db: Surreal<Any>,
    connection: Connection,
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("surrealdb: {0}")]
    Surreal(#[from] surrealdb::Error),
    #[error("response decode: {0}")]
    Decode(String),
}

impl Client {
    pub async fn connect(connection: Connection) -> Result<Self, ClientError> {
        let db = surrealdb::engine::any::connect(&connection.url).await?;
        db.signin(Root {
            username: connection.user.clone(),
            password: connection.pass.clone(),
        })
        .await?;
        Ok(Self { db, connection })
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn db(&self) -> &Surreal<Any> {
        &self.db
    }

    /// Switch the connection's namespace and database. Run once after the
    /// bootstrap query has DEFINE'd them.
    pub async fn use_ns_db(&self) -> Result<(), ClientError> {
        self.db
            .use_ns(&self.connection.namespace)
            .use_db(&self.connection.database)
            .await?;
        Ok(())
    }

    /// Run multi-statement SurrealQL and surface any per-statement error as a
    /// single `ClientError`. The caller is responsible for having already
    /// scoped the connection (or for embedding `USE NS .. DB ..` in `sql`).
    pub async fn run_sql(&self, sql: &str) -> Result<(), ClientError> {
        self.db.query(sql).await?.check()?;
        Ok(())
    }
}
