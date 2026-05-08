//! Shared `aurora.toml` schema and loader.
//!
//! Every Aurora tool that needs to know where the schema lives, where
//! migrations go, or how to talk to a database, reads the same `aurora.toml`
//! through this crate. Keeping it in its own crate lets future tools depend
//! on the config without pulling in heavier crates like aurora-migrate.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("io error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid config at {path}: {message}")]
    Invalid { path: String, message: String },
}

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub schema: SchemaConfig,
    #[serde(default)]
    pub migrations: MigrationsConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchemaConfig {
    #[serde(default = "default_schema_file")]
    pub file: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MigrationsConfig {
    #[serde(default = "default_migrations_dir")]
    pub dir: PathBuf,
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Self {
            file: default_schema_file(),
        }
    }
}

impl Default for MigrationsConfig {
    fn default() -> Self {
        Self {
            dir: default_migrations_dir(),
        }
    }
}

/// Names of env vars to read for SurrealDB connection. Each field, if set,
/// overrides the default name cascade with a single explicit name (no
/// fallback). If a field is `None`, the applier walks the default cascade
/// (e.g. `SURREALDB_PASS` → `SURREALDB_PASSWORD` → `SURREAL_PASS` → ...).
///
/// Values are never stored in this file — only the *names* of env vars.
/// Credentials live in `.env` (or `.env.local`, or the deploy environment).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub pass: Option<String>,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub database: Option<String>,
    /// Optional path (relative to `aurora.toml`'s directory, or absolute) to
    /// a dotenv file to load. If unset, we look for `.env` first and fall
    /// back to `.env.local` in the same directory as `aurora.toml`.
    #[serde(default)]
    pub env_path: Option<PathBuf>,
}

pub fn default_schema_file() -> PathBuf {
    PathBuf::from("schema.aurora")
}

pub fn default_migrations_dir() -> PathBuf {
    PathBuf::from("migrations")
}

/// Walk up from `start` looking for `aurora.toml`. Returns the parsed
/// config; falls back to `Config::default()` if no `aurora.toml` is found.
pub fn load(start: &Path) -> Result<Config> {
    Ok(load_with_dir(start)?.0)
}

/// Like `load`, but also returns the directory that contains `aurora.toml`
/// (or `start` if none was found). Callers that resolve relative paths
/// (`migrations.dir`, `.env.local`, etc.) want the dir.
pub fn load_with_dir(start: &Path) -> Result<(Config, PathBuf)> {
    let mut current = if start.is_file() {
        start.parent().unwrap_or(start).to_path_buf()
    } else {
        start.to_path_buf()
    };

    loop {
        let candidate = current.join("aurora.toml");
        if candidate.exists() {
            let config = load_file(&candidate)?;
            return Ok((config, current));
        }
        if !current.pop() {
            return Ok((Config::default(), start.to_path_buf()));
        }
    }
}

pub fn load_file(path: &Path) -> Result<Config> {
    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.display().to_string(),
        source,
    })?;
    toml::from_str(&contents).map_err(|error| ConfigError::Invalid {
        path: path.display().to_string(),
        message: error.to_string(),
    })
}
