// Shared syntax strings used by recovery diagnostics and lightweight parser
// context checks. Keeping them here prevents those layers from drifting as
// declarations grow.
pub(super) const ANALYZER: &str = "analyzer";
pub(super) const TABLE: &str = "table";

pub(super) const TOP_LEVEL_DECLARATIONS: &[&str] = &[TABLE, ANALYZER];
