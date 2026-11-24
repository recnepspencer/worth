use crate::history::retention::{
    RelationalBranchRetentionLease, RelationalBranchRetentionReleaseReceipt,
};
use crate::mvcc::observation::RelationalBranchObservation;
use crate::snapshots::data::SnapshotId;
pub mod retirement;
pub use retirement::{
    RelationalRetainedObservationRetirement, RelationalRetainedObservationRetirementDenial,
    RelationalRetainedObservationRetirementProgress, RetiredRelationalRetainedObservation,
};

/// One admitted basis, retained and given its own snapshot id.
///
/// The lease keeps the basis's commit history alive until it is released or
/// dropped. The snapshot id is unique within the issuing runtime, so a
/// consumer can key its own tables by it.
#[derive(Debug)]
#[must_use = "dropping the lease ends the retention it holds"]
pub struct RelationalRetainedObservation {
    snapshot_id: SnapshotId,
    observation: RelationalBranchObservation,
    retention: RelationalBranchRetentionLease,
}

impl RelationalRetainedObservation {
    /// The snapshot id allocated for this retention.
    pub fn snapshot_id(&self) -> SnapshotId {
        self.snapshot_id
    }

    /// The repeatable read view of the retained basis.
    pub fn observation(&self) -> &RelationalBranchObservation {
        &self.observation
    }

    /// End the retention exactly once and report how it ended.
    pub fn release(self) -> RelationalBranchRetentionReleaseReceipt {
        self.retention.release()
    }
}

/// Operations used by the Relational owners.
pub trait RelationalRetainedObservationOwnerPort: Sized {
    fn from_admitted(
        snapshot_id: SnapshotId,
        observation: RelationalBranchObservation,
        retention: RelationalBranchRetentionLease,
    ) -> Self;
}

impl RelationalRetainedObservationOwnerPort for RelationalRetainedObservation {
    fn from_admitted(
        snapshot_id: SnapshotId,
        observation: RelationalBranchObservation,
        retention: RelationalBranchRetentionLease,
    ) -> Self {
        Self {
            snapshot_id,
            observation,
            retention,
        }
    }
}
