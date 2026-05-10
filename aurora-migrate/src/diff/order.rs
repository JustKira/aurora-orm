use crate::ops::Op;

/// Final sequencing pass over the flat op list produced by per-entity differs.
///
/// Today this is the identity function: each differ already emits its ops in
/// the order the renderer expects (CreateTable → its CreateIndexes; analyzers
/// before tables that reference them via fulltext indexes purely by virtue of
/// orchestrator dispatch order).
///
/// Future home for cross-entity ordering rules — e.g. when events/accesses
/// land, "RemoveIndex before RemoveField that backed it" or "CreateAccess
/// after CreateTable it scopes". Promote to a real toposort when priority-
/// by-kind alone stops being sufficient.
pub(crate) fn sort_ops(ops: Vec<Op>) -> Vec<Op> {
    ops
}
