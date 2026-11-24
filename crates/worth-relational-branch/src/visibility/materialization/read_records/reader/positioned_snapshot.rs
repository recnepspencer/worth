use crate::history::data::{BranchId, CommitId};
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;

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

pub struct PositionedRelationalSnapshotParts {
    pub runtime_instance_id: u64,
    pub branch_id: BranchId,
    pub root_id: u64,
    pub version_id: VersionId,
    pub commit_id: Option<CommitId>,
    pub position: Option<PatchStreamPosition>,
}

pub fn assemble_positioned_relational_snapshot(
    parts: PositionedRelationalSnapshotParts,
) -> PositionedRelationalSnapshot {
    PositionedRelationalSnapshot {
        runtime_instance_id: parts.runtime_instance_id,
        branch_id: parts.branch_id,
        root_id: parts.root_id,
        version_id: parts.version_id,
        commit_id: parts.commit_id,
        position: parts.position,
    }
}

pub trait PositionedRelationalSnapshotFieldsPort {
    fn owner_branch_id(&self) -> &BranchId;
    fn owner_commit_id_mut(&mut self) -> &mut Option<CommitId>;
    fn owner_commit_id(&self) -> &Option<CommitId>;
    fn owner_position_mut(&mut self) -> &mut Option<PatchStreamPosition>;
    fn owner_position(&self) -> &Option<PatchStreamPosition>;
    fn owner_root_id_mut(&mut self) -> &mut u64;
    fn owner_root_id(&self) -> &u64;
    fn owner_runtime_instance_id_mut(&mut self) -> &mut u64;
    fn owner_runtime_instance_id(&self) -> &u64;
}

impl PositionedRelationalSnapshotFieldsPort for PositionedRelationalSnapshot {
    fn owner_branch_id(&self) -> &BranchId {
        &self.branch_id
    }
    fn owner_commit_id_mut(&mut self) -> &mut Option<CommitId> {
        &mut self.commit_id
    }
    fn owner_commit_id(&self) -> &Option<CommitId> {
        &self.commit_id
    }
    fn owner_position_mut(&mut self) -> &mut Option<PatchStreamPosition> {
        &mut self.position
    }
    fn owner_position(&self) -> &Option<PatchStreamPosition> {
        &self.position
    }
    fn owner_root_id_mut(&mut self) -> &mut u64 {
        &mut self.root_id
    }
    fn owner_root_id(&self) -> &u64 {
        &self.root_id
    }
    fn owner_runtime_instance_id_mut(&mut self) -> &mut u64 {
        &mut self.runtime_instance_id
    }
    fn owner_runtime_instance_id(&self) -> &u64 {
        &self.runtime_instance_id
    }
}

impl PositionedRelationalSnapshot {
    #[doc(hidden)]
    pub(crate) fn append_foreign_branch_suffix_for_test(&mut self, suffix: &str) {
        self.branch_id.0.push_str(suffix);
    }
}

#[doc(hidden)]
pub trait PositionedRelationalSnapshotFixturePort: Sized {
    fn append_foreign_branch_suffix_for_test(&mut self, suffix: &str);
}

impl PositionedRelationalSnapshotFixturePort for PositionedRelationalSnapshot {
    fn append_foreign_branch_suffix_for_test(&mut self, suffix: &str) {
        self.append_foreign_branch_suffix_for_test(suffix)
    }
}
