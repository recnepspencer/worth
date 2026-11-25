use serde::{Deserialize, Serialize};

use crate::history::data::BranchId;
use crate::identity::data::VersionId;

pub use coordinates::{SnapshotId, SnapshotReadPolicy};
pub use worth_relational_contracts::internal_port::snapshots::data::coordinates;

/// Owner-issued operational capability for one retained snapshot binding.
///
/// The fields and constructor are crate-private so copied identifiers and
/// transported data cannot be promoted into snapshot authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotHandle {
    pub(crate) runtime_instance_id: u64,
    pub(crate) branch_id: BranchId,
    pub(crate) snapshot_id: SnapshotId,
    pub(crate) version_id: VersionId,
    pub(crate) read_policy: SnapshotReadPolicy,
}

impl SnapshotHandle {
    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub const fn snapshot_id(&self) -> SnapshotId {
        self.snapshot_id
    }

    pub const fn version_id(&self) -> VersionId {
        self.version_id
    }

    pub const fn read_policy(&self) -> SnapshotReadPolicy {
        self.read_policy
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotInspectionSummary {
    pub branch_id: BranchId,
    pub version_id: VersionId,
    /// Owner-issued immutable root identity selected by this snapshot.
    /// `None` is reserved for the empty-runtime bootstrap.
    pub root_id: Option<u64>,
    pub entity_count: usize,
    pub relation_count: usize,
    pub pinned_entity_count: usize,
    pub pinned_relation_count: usize,
}

pub trait SnapshotHandleFieldsPort {
    fn owner_read_policy(&self) -> &SnapshotReadPolicy;
    fn owner_runtime_instance_id(&self) -> &u64;
    fn owner_snapshot_id(&self) -> &SnapshotId;
    fn owner_version_id(&self) -> &VersionId;
    fn owner_branch_id(&self) -> &BranchId;
}

impl SnapshotHandleFieldsPort for SnapshotHandle {
    fn owner_read_policy(&self) -> &SnapshotReadPolicy {
        &self.read_policy
    }
    fn owner_runtime_instance_id(&self) -> &u64 {
        &self.runtime_instance_id
    }
    fn owner_snapshot_id(&self) -> &SnapshotId {
        &self.snapshot_id
    }
    fn owner_version_id(&self) -> &VersionId {
        &self.version_id
    }
    fn owner_branch_id(&self) -> &BranchId {
        &self.branch_id
    }
}

pub fn assemble_snapshot_handle(
    runtime_instance_id: u64,
    branch_id: BranchId,
    snapshot_id: SnapshotId,
    version_id: VersionId,
    read_policy: SnapshotReadPolicy,
) -> SnapshotHandle {
    SnapshotHandle {
        runtime_instance_id,
        branch_id,
        snapshot_id,
        version_id,
        read_policy,
    }
}

#[doc(hidden)]
pub trait SnapshotHandleFixturePort {
    fn replace_snapshot_version_for_test(&mut self, version: VersionId);
}

impl SnapshotHandleFixturePort for SnapshotHandle {
    fn replace_snapshot_version_for_test(&mut self, version: VersionId) {
        self.version_id = version;
    }
}
