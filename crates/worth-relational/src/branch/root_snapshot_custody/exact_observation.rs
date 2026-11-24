//! Capture and retain an exact root at the observation owner's boundary.
use crate::visibility::materialization::read_records::projection::{
    ProjectionInputs, VisibilityProjectionView,
};

impl VisibilityProjectionView {
    pub(crate) fn retain_exact_basis(
        &self,
    ) -> Option<crate::history::retention::VisibilitySnapshotBasis> {
        match self.snapshot_basis() {
            crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis) => {
                Some(basis.clone())
            }
            crate::visibility::snapshot_states::SnapshotStateBasis::Historical(_) => None,
        }
    }
}

#[path = "exact_observation/runtime_reads.rs"]
mod runtime_reads;

impl VisibilityProjectionView {
    pub(crate) fn from_observation(
        inputs: ProjectionInputs<'_>,
        observation: &crate::mvcc::observation::RelationalBranchObservation,
    ) -> Result<Self, crate::branch::basis_denial::RelationalBranchBasisDenial> {
        if observation.identity().runtime_instance_id() != inputs.runtime_identity {
            return Err(
                crate::branch::basis_denial::RelationalBranchBasisDenial::ForeignRuntime {
                    expected_runtime_instance_id: inputs.runtime_identity,
                    actual_runtime_instance_id: observation.identity().runtime_instance_id(),
                },
            );
        }
        if observation
            .selected_root()
            .has_materialization_unavailable()
        {
            return Err(crate::branch::basis_denial::RelationalBranchBasisDenial::MaterializationUnavailable);
        }
        let basis =
            crate::history::retention::VisibilitySnapshotBasis::from_observation(observation);
        Ok(VisibilityProjectionView::new(
            inputs,
            crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis),
        ))
    }
}
