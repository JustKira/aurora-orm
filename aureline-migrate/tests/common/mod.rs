#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use aureline_core::ast::{Field, Schema, SchemaItem, Table, Type};
use aureline_migrate::config::{Config, MigrationsConfig, SchemaConfig};

pub fn schema(tables: Vec<Table>) -> Schema {
    Schema {
        items: tables.into_iter().map(SchemaItem::TableDecl).collect(),
    }
}

pub fn table(name: &str, modifier: Option<&str>, fields: Vec<Field>) -> Table {
    Table {
        name: name.to_string(),
        modifier: modifier.map(str::to_string),
        fields,
        indexes: Vec::new(),
        raw_attributes: Vec::new(),
    }
}

pub fn field(name: &str, type_name: &str, optional: bool) -> Field {
    Field {
        name: name.to_string(),
        ty: Type::primitive(type_name),
        optional,
        flexible: false,
        raw_attributes: Vec::new(),
    }
}

pub fn config(schema: PathBuf, migrations: PathBuf) -> Config {
    Config {
        schema: SchemaConfig { file: schema },
        migrations: MigrationsConfig { dir: migrations },
        database: Default::default(),
    }
}

pub fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("aureline_migrate_{label}_{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}
