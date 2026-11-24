use serde::{Deserialize, Serialize};

use crate::history::data::BranchId;
use crate::identity::data::VersionId;

pub(crate) mod coordinates;
pub use coordinates::{SnapshotId, SnapshotReadPolicy};

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
