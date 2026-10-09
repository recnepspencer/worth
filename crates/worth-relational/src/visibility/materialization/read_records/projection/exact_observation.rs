use super::super::reader::VisibilityReadContext;
use super::VisibilityProjectionView;

impl VisibilityProjectionView<'_> {
    pub(crate) fn retain_exact_basis(
        &self,
    ) -> Option<crate::visibility::snapshot_states::VisibilitySnapshotBasis> {
        match &self.basis {
            crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis) => {
                Some(basis.clone())
            }
            crate::visibility::snapshot_states::SnapshotStateBasis::Historical(_) => None,
        }
    }
}

impl<'runtime> VisibilityReadContext<'runtime> {
    /// Project through an owner-issued exact branch observation without
    /// allocating a tracked snapshot handle.
    pub fn project_observation(
        &self,
        observation: &crate::mvcc::RelationalBranchObservation,
    ) -> Result<VisibilityProjectionView<'runtime>, crate::branch::RelationalBranchBasisDenial>
    {
        if observation.identity().runtime_instance_id() != self.runtime().runtime_instance_id() {
            return Err(crate::branch::RelationalBranchBasisDenial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime().runtime_instance_id(),
                actual_runtime_instance_id: observation.identity().runtime_instance_id(),
            });
        }
        if observation
            .selected_root()
            .has_materialization_unavailable()
        {
            return Err(crate::branch::RelationalBranchBasisDenial::MaterializationUnavailable);
        }
        let basis = crate::visibility::snapshot_states::VisibilitySnapshotBasis::from_observation(
            observation,
        );
        Ok(VisibilityProjectionView::new(
            self.runtime(),
            crate::visibility::snapshot_states::SnapshotStateBasis::Exact(basis),
        ))
    }
}
