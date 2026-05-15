#![allow(dead_code, unused_imports, unused_macros)]

use std::path::PathBuf;

use aureline_core::ast::Schema;
use aureline_migrate::config::{Config, MigrationsConfig, SchemaConfig};
use aureline_migrate::diff::diff_schemas;
use aureline_migrate::render::{emit_down, emit_up};

macro_rules! expected_surql {
    ($($line:literal),* $(,)?) => {
        aureline_test_support::expected_surql!($($line),*)
    };
}

macro_rules! aureline_schema {
    ($($line:literal),* $(,)?) => {
        aureline_test_support::aureline_schema!($($line),*)
    };
}

pub use aureline_test_support::{empty_schema, field, parse_schema, schema, table, temp_dir};

pub fn diff_up(prev: &Schema, next: &Schema) -> String {
    emit_up(&diff_schemas(prev, next))
}

pub fn diff_down(prev: &Schema, next: &Schema) -> String {
    emit_down(&diff_schemas(prev, next))
}

pub fn config(schema: PathBuf, migrations: PathBuf) -> Config {
    Config {
        schema: SchemaConfig { file: schema },
        migrations: MigrationsConfig { dir: migrations },
        database: Default::default(),
    }
}
