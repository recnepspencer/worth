use crate::history::data::coordinates::BranchId;
use serde::{Deserialize, Serialize};

/// Denial returned when the runtime cannot bind a descriptive merge selector
/// to an exact owner-owned branch cell.  A descriptive `BranchId` is never
/// sufficient to enter planning or execution by itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeRequestBindingDenial {
    ForeignRuntime {
        expected_runtime_instance_id: u64,
        actual_runtime_instance_id: u64,
    },
    UnknownBranch(BranchId),
    ArchivedBranch(BranchId),
    DeletingBranch(BranchId),
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    IdentityMismatch,
}
