//! Admit snapshot identity for retained Native observations.
use super::retained_observation::RelationalRetainedObservation;
use crate::branch::{AdmittedRelationalBranchBasis, RelationalBranchBasisDenial};
use crate::runtime::RelationalRuntime;
use worth_relational_branch::internal_port::change_source::retained_observation::RelationalRetainedObservationOwnerPort as _;

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
        Ok(RelationalRetainedObservation::from_admitted(
            snapshot_id,
            basis.observation(),
            retention,
        ))
    }
}
