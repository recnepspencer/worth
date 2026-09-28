use crate::branch::{AdmittedRelationalBranchBasis, RelationalBranchBasisDenial};
use crate::history::retention::{
    RelationalBranchRetentionLease, RelationalBranchRetentionReleaseReceipt,
};
use crate::mvcc::RelationalBranchObservation;
use crate::runtime::RelationalRuntime;
use crate::snapshots::data::SnapshotId;

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

impl RelationalRuntime {
    /// Retain `basis` and allocate its snapshot id, in that order, as one
    /// operation.
    ///
    /// # Errors
    ///
    /// Returns the basis denial when `basis` belongs to another runtime or
    /// can no longer be retained, and
    /// [`RelationalBranchBasisDenial::SnapshotIdentityExhausted`] when the
    /// runtime has no snapshot id left; the retention is then dropped.
    pub fn retain_observation_snapshot(
        &self,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<RelationalRetainedObservation, RelationalBranchBasisDenial> {
        let retention = self.retain_component_basis(basis)?;
        let snapshot_id = self
            .visibility
            .allocate_snapshot_id()
            .ok_or(RelationalBranchBasisDenial::SnapshotIdentityExhausted)?;
        Ok(RelationalRetainedObservation {
            snapshot_id,
            observation: basis.observation(),
            retention,
        })
    }
}
