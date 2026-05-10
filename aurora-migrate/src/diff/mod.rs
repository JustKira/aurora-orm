use aurora_core::ast::Schema;

use crate::ops::Op;

mod analyzer;
mod field;
mod index;
mod order;
mod pair;
mod table;

pub fn diff_schemas(prev: &Schema, new: &Schema) -> Vec<Op> {
    let mut ops = Vec::new();
    analyzer::diff_analyzers(prev, new, &mut ops);
    table::diff_tables(prev, new, &mut ops);
    order::sort_ops(ops)
}
