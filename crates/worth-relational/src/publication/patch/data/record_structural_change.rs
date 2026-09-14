use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RecordStructuralChange {
    Created,
    Updated,
    Deleted,
    RetainedForAudit,
    MaterializationSuspended,
    Rematerialized,
}
