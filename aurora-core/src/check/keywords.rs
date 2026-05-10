// Shared syntax strings used by diagnostics. Keeping them here prevents the
// classifier/context layers from drifting as top-level declarations grow.
pub(super) const ANALYZER: &str = "analyzer";
pub(super) const DOC_COMMENT: &str = "///";
pub(super) const TABLE: &str = "table";

pub(super) const TOP_LEVEL_DECLARATIONS: &[&str] = &[TABLE, ANALYZER];
