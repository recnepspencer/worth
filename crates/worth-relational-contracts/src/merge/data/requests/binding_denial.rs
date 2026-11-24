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

impl From<RelationalMergeRequestBindingDenial>
    for crate::branch::basis_denial::RelationalBranchBasisDenial
{
    fn from(value: RelationalMergeRequestBindingDenial) -> Self {
        match value {
            RelationalMergeRequestBindingDenial::ForeignRuntime {
                expected_runtime_instance_id,
                actual_runtime_instance_id,
            } => Self::ForeignRuntime {
                expected_runtime_instance_id,
                actual_runtime_instance_id,
            },
            RelationalMergeRequestBindingDenial::UnknownBranch(branch) => {
                Self::UnknownBranch(branch)
            }
            RelationalMergeRequestBindingDenial::ArchivedBranch(branch) => {
                Self::ArchivedBranch(branch)
            }
            RelationalMergeRequestBindingDenial::DeletingBranch(branch) => {
                Self::DeletingBranch(branch)
            }
            RelationalMergeRequestBindingDenial::RetentionCapacityExhausted => {
                Self::RetentionCapacityExhausted
            }
            RelationalMergeRequestBindingDenial::RetentionIdentityExhausted => {
                Self::RetentionIdentityExhausted
            }
            RelationalMergeRequestBindingDenial::SnapshotIdentityExhausted => {
                Self::SnapshotIdentityExhausted
            }
            RelationalMergeRequestBindingDenial::IdentityMismatch => Self::MixedAxis(
                crate::branch::basis_denial::RelationalBranchBasisMismatchAxis::Branch,
            ),
        }
    }
}
