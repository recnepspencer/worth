//! Descriptive outcomes for retiring a commit result's original snapshot.
use crate::visibility::snapshot_release::{
    RelationalSnapshotReleaseDenial, RelationalSnapshotReleaseReceipt,
};

#[derive(Debug)]
pub enum RelationalCommitResultSnapshotRelease {
    Released(RelationalSnapshotReleaseReceipt),
    AlreadyReleased {
        snapshot_id: crate::snapshots::data::coordinates::SnapshotId,
    },
}
#[derive(Debug)]
pub enum RelationalCommitResultSnapshotReleaseDenial {
    Owner(crate::branch::basis_denial::RelationalBranchBasisDenial),
    Snapshot(RelationalSnapshotReleaseDenial),
    ResultBindingMismatch,
    RegistrationMismatch,
    RetirementInterrupted,
}
#[derive(Debug)]
pub enum RelationalCommitResultSnapshotCleanupPending {
    Advanced,
    Busy,
    Stopped(worth_execution::WorkCeilingDenial),
}

#[derive(Debug)]
pub enum RelationalCommitResultSnapshotCleanupDenial {
    NotOwner,
    OwnerBindingMismatch,
    Release(RelationalCommitResultSnapshotReleaseDenial),
}
