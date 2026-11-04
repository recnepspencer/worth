use std::sync::Arc;

use super::RelationalBranchReferenceCell;

impl RelationalBranchReferenceCell {
    pub(crate) fn root(&self) -> Option<Arc<super::super::RelationalBranchRoot>> {
        self.state().root.clone()
    }

    pub(crate) fn install_root(&mut self, root: Arc<super::super::RelationalBranchRoot>) {
        self.state().root = Some(root);
    }

    pub(crate) fn register_basis(
        &self,
        basis: super::super::AdmittedRelationalBranchBasis,
    ) -> Result<
        super::super::AdmittedRelationalBranchBasis,
        super::super::RelationalBranchBasisDenial,
    > {
        self.basis_registry.register(basis)
    }

    pub(crate) fn retained_bases_for_request<E>(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        pay_scan: impl FnOnce(usize) -> Result<(), E>,
    ) -> Result<
        worth_execution::ExecutionArray<
            std::sync::Weak<super::super::AdmittedRelationalBranchBasisInner>,
        >,
        super::super::RelationalRetainedBasisSnapshotStop<E>,
    > {
        self.basis_registry
            .snapshot_retained_for_request(request, pay_scan)
    }

    pub(crate) fn readmit_retained_basis(
        &self,
        descriptor: &super::super::RelationalBranchBasisDescriptor,
    ) -> Option<super::super::AdmittedRelationalBranchBasis> {
        self.basis_registry.readmit_retained(descriptor)
    }

    pub(crate) fn bind_basis_registry_metrics(
        &mut self,
        metrics: Arc<super::super::RelationalBranchBasisRegistryMetrics>,
    ) {
        self.basis_registry.bind_metrics(metrics);
    }

    pub(crate) fn reset_basis_registry(
        &mut self,
        metrics: Arc<super::super::RelationalBranchBasisRegistryMetrics>,
    ) {
        self.basis_registry = super::super::RelationalBranchBasisRegistry::with_metrics(metrics);
    }

    #[cfg(test)]
    pub(crate) fn clear_root_for_test(&mut self) {
        self.state().root = None;
    }
}

/// Shallow original cell DATA custody. Capturing this handle takes no state or
/// head lock and copies no name, descriptor, publication handle or service.
pub(crate) struct CheckpointComponentCellSource {
    runtime_instance_id: u64,
    state: Arc<std::sync::Mutex<super::RelationalBranchReferenceMutableState>>,
    head: Arc<crate::history::retention::RelationalBranchHeadRetentionCell>,
}
impl RelationalBranchReferenceCell {
    pub(in crate::branch) fn checkpoint_component_source(&self) -> CheckpointComponentCellSource {
        CheckpointComponentCellSource {
            runtime_instance_id: self.identity.runtime_instance_id(),
            state: Arc::clone(&self.state),
            head: Arc::clone(&self.head_retention),
        }
    }
}
impl CheckpointComponentCellSource {
    /// Fixed original-source DATA; registry ownership was released before this borrow.
    pub(crate) fn fresh_empty_root(
        &self,
    ) -> Option<(
        worth_foundational::FoundationalBranchReferenceGeneration,
        super::RelationalBranchVersion,
        Arc<super::super::RelationalBranchRoot>,
    )> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if !matches!(
            state.observation.target(),
            worth_foundational::FoundationalBranchTarget::Empty
        ) || state.lifecycle != super::super::RelationalBranchLifecyclePosture::Live
        {
            return None;
        }
        Some((
            state.observation.generation(),
            state.truth_version,
            Arc::clone(state.root.as_ref()?),
        ))
    }

    pub(crate) fn snapshot(
        &self,
        expected: &crate::branch::RelationalBranchCellCheckpoint,
        posture: crate::branch::RelationalBranchLifecyclePosture,
    ) -> Result<
        Option<(
            Arc<crate::branch::RelationalBranchRoot>,
            crate::history::retention::RelationalBranchRetentionBinding,
        )>,
        crate::branch::CheckpointBranchMetadataCaptureDenial,
    > {
        use crate::branch::CheckpointBranchMetadataCaptureDenial as Denial;
        let root = {
            let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            if self.runtime_instance_id != expected.runtime_instance_id
                || state.observation != expected.observation
                || state.truth_version != expected.truth_version
                || state.lifecycle != posture
            {
                return Err(Denial::SourceChanged);
            }
            let Some(root) = state.root.as_ref() else {
                return Ok(None);
            };
            super::super::basis_axis_validation::require_root_matches_reference(
                &state.observation,
                root,
            )
            .map_err(|_| Denial::SourceChanged)?;
            Arc::clone(root)
        };
        // Never enter the head/lane while holding the actual state guard.
        let binding = self.head.binding().map_err(|_| Denial::SourceChanged)?;
        Ok(Some((root, binding)))
    }
}
