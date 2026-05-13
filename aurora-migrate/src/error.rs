#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("parse error in {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: aurora_core::AuroraError,
    },

    #[error("snapshot decode error: {0}")]
    SnapshotDecode(String),

    #[error("invalid journal: {0}")]
    Journal(String),

    #[error(transparent)]
    Config(#[from] aurora_config::ConfigError),

    #[error("invalid migration lockfile at {path}: {message}")]
    Lock { path: String, message: String },

    #[error("schema unchanged - nothing to generate (use --allow-empty to override)")]
    EmptyDiff,

    #[error("invalid migration name '{name}': {reason}")]
    InvalidName { name: String, reason: String },

    #[error("unsupported schema item in {path}: {message}")]
    UnsupportedSchemaItem { path: String, message: String },
}

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn io(path: impl AsRef<std::path::Path>, source: std::io::Error) -> Error {
    Error::Io {
        path: path.as_ref().display().to_string(),
        source,
    }
}
