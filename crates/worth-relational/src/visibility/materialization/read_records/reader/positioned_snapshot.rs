use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::snapshots::data::SnapshotHandle;

/// The canonical position of the exact immutable root selected by a snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionedRelationalSnapshot {
    pub(crate) runtime_instance_id: u64,
    pub(crate) branch_id: BranchId,
    pub(crate) root_id: u64,
    pub(crate) version_id: VersionId,
    pub(crate) commit_id: Option<CommitId>,
    pub(crate) position: Option<PatchStreamPosition>,
}

impl PositionedRelationalSnapshot {
    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }
    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }
    pub const fn root_id(&self) -> u64 {
        self.root_id
    }
    pub const fn version_id(&self) -> VersionId {
        self.version_id
    }
    pub const fn commit_id(&self) -> Option<CommitId> {
        self.commit_id
    }
    pub const fn position(&self) -> Option<PatchStreamPosition> {
        self.position
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotPositionDenial {
    SnapshotUnavailable,
    SelectedCommitPositionUnavailable { commit_id: CommitId },
}

#[derive(Debug)]
pub enum RelationalSnapshotPositionAdmissionStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
    Position(SnapshotPositionDenial),
}

#[path = "positioned_snapshot/runtime_reads.rs"]
mod runtime_reads;
