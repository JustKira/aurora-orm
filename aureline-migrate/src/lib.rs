pub mod apply;
pub mod change;
pub mod checksum;
pub mod diff;
pub mod error;
pub mod fs_io;
mod generate;
pub mod journal;
pub mod ops;
pub mod plan;
pub mod render;
pub mod schema;
pub mod snapshot;

pub use apply::{ApplyOpts, ApplyReport, apply};
pub use generate::{GenerateOpts, GenerateReport, generate};

/// Re-export the shared config module so existing callers can still write
/// `aureline_migrate::config::Config`. New code can depend on `aureline-config`
/// directly.
pub use aureline_config as config;
