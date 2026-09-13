use crate::state::{SignalBranchHandle, SignalSnapshotId};

use super::branches::BranchState;
use super::SignalBranchBasisArtifact;

pub(super) struct ResolvedForkRequest<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(super) parent_branch: SignalBranchHandle,
    #[cfg(test)]
    pub(super) parent_basis: SignalBranchBasisArtifact,
    pub(super) requested_snapshot_basis: Option<SignalBranchBasisArtifact>,
    pub(super) created_branch_head_snapshot_id: Option<SignalSnapshotId>,
    pub(super) source_branch_state: BranchState<D, I, T>,
}
