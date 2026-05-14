use aureline_core::ast::{Field, Table};

#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    TableAdded(Table),
    TableRemoved(String),
    TableModeChanged {
        table: String,
        from: Option<String>,
        to: Option<String>,
    },
    FieldAdded {
        table: String,
        field: Field,
    },
    FieldRemoved {
        table: String,
        field: Field,
    },
    FieldChanged {
        table: String,
        from: Field,
        to: Field,
        changes: FieldChangeSet,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FieldChangeSet {
    pub type_changed: bool,
    pub optional_changed: bool,
    pub flexible_changed: bool,
}

impl FieldChangeSet {
    pub fn between(from: &Field, to: &Field) -> Self {
        Self {
            type_changed: from.ty != to.ty,
            optional_changed: from.optional != to.optional,
            flexible_changed: from.flexible != to.flexible,
        }
    }

    pub fn is_empty(self) -> bool {
        !self.type_changed && !self.optional_changed && !self.flexible_changed
    }
}
