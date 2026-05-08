use std::path::Path;

use aurora_config::DatabaseConfig;

#[derive(Debug, thiserror::Error)]
pub enum EnvError {
    #[error("missing required env var: tried {0:?}; configure one of these or override via aurora.toml [database]")]
    Missing(Vec<String>),
    #[error("env var {name} (set in aurora.toml) is not defined in the environment")]
    OverrideMissing { name: String },
    #[error("invalid URL in env var {name}: {value}")]
    InvalidUrl { name: String, value: String },
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub url: String,
    pub user: String,
    pub pass: String,
    pub namespace: String,
    pub database: String,
}

/// Resolve SurrealDB connection params from env vars.
///
/// Loads dotenv variables from (in priority order):
///   1. `db.env_path` if set in aurora.toml (relative to `config_dir`).
///   2. `<config_dir>/.env` if it exists.
///   3. `<config_dir>/.env.local` if it exists.
///
/// Only one of these is loaded — the first match wins. Then resolves each
/// connection field using either the override name from aurora.toml or the
/// default name cascade.
pub fn resolve_connection(
    db: &DatabaseConfig,
    config_dir: &Path,
) -> Result<Connection, EnvError> {
    load_dotenv(config_dir, db.env_path.as_deref());

    Ok(Connection {
        url: resolve_field(&db.url, &URL_DEFAULTS, "url")?,
        user: resolve_field(&db.user, &USER_DEFAULTS, "user")?,
        pass: resolve_field(&db.pass, &PASS_DEFAULTS, "pass")?,
        namespace: resolve_field(&db.namespace, &NS_DEFAULTS, "namespace")?,
        database: resolve_field(&db.database, &DB_DEFAULTS, "database")?,
    })
}

const URL_DEFAULTS: &[&str] = &["SURREALDB_URL", "SURREAL_URL"];
const USER_DEFAULTS: &[&str] = &["SURREALDB_USER", "SURREAL_USER"];
const PASS_DEFAULTS: &[&str] = &[
    "SURREALDB_PASS",
    "SURREALDB_PASSWORD",
    "SURREAL_PASS",
    "SURREAL_PASSWORD",
];
const NS_DEFAULTS: &[&str] = &[
    "SURREALDB_NS",
    "SURREALDB_NAMESPACE",
    "SURREAL_NS",
    "SURREAL_NAMESPACE",
];
const DB_DEFAULTS: &[&str] = &[
    "SURREALDB_DB",
    "SURREALDB_DATABASE",
    "SURREAL_DB",
    "SURREAL_DATABASE",
];

fn resolve_field(
    override_name: &Option<String>,
    defaults: &[&str],
    _label: &str,
) -> Result<String, EnvError> {
    if let Some(name) = override_name {
        return std::env::var(name)
            .map_err(|_| EnvError::OverrideMissing { name: name.clone() });
    }
    for name in defaults {
        if let Ok(value) = std::env::var(name) {
            return Ok(value);
        }
    }
    Err(EnvError::Missing(
        defaults.iter().map(|s| s.to_string()).collect(),
    ))
}

/// Load a single dotenv file. Resolution order: explicit `override_path` (from
/// `aurora.toml`'s `[database] env_path`) → `<dir>/.env` → `<dir>/.env.local`.
/// First match wins; if none exist, do nothing and let the process inherit
/// whatever env it was launched with.
fn load_dotenv(dir: &Path, override_path: Option<&Path>) {
    if let Some(rel) = override_path {
        let path = if rel.is_absolute() {
            rel.to_path_buf()
        } else {
            dir.join(rel)
        };
        // `from_path` (not `from_path_override`) so pre-set process env vars
        // win over the file. Lets a CLI invocation override an .env value.
        let _ = dotenvy::from_path(&path);
        return;
    }
    for name in [".env", ".env.local"] {
        let path = dir.join(name);
        if path.exists() {
            // `from_path` (not `from_path_override`) so pre-set process env vars
        // win over the file. Lets a CLI invocation override an .env value.
        let _ = dotenvy::from_path(&path);
            return;
        }
    }
}
