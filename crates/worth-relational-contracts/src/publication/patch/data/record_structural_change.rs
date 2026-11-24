use serde::{Deserialize, Serialize};

/// How a commit changed one record's structural presence, as reported in a
/// published patch.
///
/// The enum is non-exhaustive: match with a wildcard arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RecordStructuralChange {
    /// The record was created.
    Created,
    /// The existing record's content changed.
    Updated,
    /// The record was deleted.
    Deleted,
    /// The relation was removed from live state but retained for audit.
    RetainedForAudit,
    /// The record's materialization was suspended.
    MaterializationSuspended,
    /// A previously suspended record was materialized again.
    Rematerialized,
}
