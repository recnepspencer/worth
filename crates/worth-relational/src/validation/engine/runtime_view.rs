use crate::identity::data::KindId;
use crate::runtime::{
    RelationalPreparationRuntime, RelationalRuntime, RuntimeInstrumentation,
    SchemaContractRuntimeSubsystem,
};

/// Read-only validation projection shared by the runtime and preparation port.
///
/// The configuration and the contract runtime lowered from it are held as the
/// one snapshot taken when the view was made, so every question this view
/// answers is answered against the same configuration even if the owner
/// installs a new one meanwhile.
#[derive(Clone)]
pub(crate) struct InvariantRuntimeView<'a> {
    pub(crate) config: std::sync::Arc<crate::runtime::RelationalRuntimeConfig>,
    pub(crate) schema_contract_runtime: std::sync::Arc<SchemaContractRuntimeSubsystem>,
    instrumentation: &'a RuntimeInstrumentation,
    shared_candidate_inputs:
        Option<std::sync::Arc<super::input_preparation::SharedCandidateInputs>>,
    #[cfg(test)]
    current_version_id: crate::identity::data::VersionId,
    entity_count: usize,
    relation_count: usize,
    version_depth: usize,
    snapshot_pressure: bool,
}

impl<'a> InvariantRuntimeView<'a> {
    pub(crate) fn from_runtime(runtime: &'a RelationalRuntime) -> Self {
        Self {
            config: std::sync::Arc::clone(&runtime.config),
            schema_contract_runtime: std::sync::Arc::clone(&runtime.schema_contract_runtime),
            instrumentation: &runtime.services.instrumentation,
            shared_candidate_inputs: None,
            #[cfg(test)]
            current_version_id: runtime.current_version_id(),
            entity_count: runtime.storage_access().entity_slot_count(),
            relation_count: runtime.storage_access().relation_slot_count(),
            version_depth: runtime.history().commit_count(),
            snapshot_pressure: runtime.visibility.active_snapshot_count() > 10,
        }
    }

    pub(crate) fn from_preparation_for_state(
        runtime: &'a RelationalPreparationRuntime,
        state: &crate::branch::RelationalBranchRootState,
    ) -> Self {
        Self {
            config: std::sync::Arc::clone(&runtime.config),
            schema_contract_runtime: std::sync::Arc::clone(&runtime.schema_contract_runtime),
            instrumentation: &runtime.services.instrumentation,
            shared_candidate_inputs: None,
            #[cfg(test)]
            current_version_id: runtime.current_version_id(),
            entity_count: state.entity_slot_count(),
            relation_count: state.relation_slot_count(),
            version_depth: runtime.current_version_id().0 as usize,
            snapshot_pressure: runtime.published_snapshot_count() > 10,
        }
    }

    pub(crate) fn performance_access(&self) -> crate::performance::PerformanceAccess<'a> {
        crate::performance::PerformanceAccess::from_instrumentation(self.instrumentation)
    }

    pub(crate) fn with_candidate_input_plan(
        &self,
        request: &super::InvariantExecutionRequest<'_>,
    ) -> Self {
        let mut view = self.clone();
        view.shared_candidate_inputs =
            super::input_preparation::SharedCandidateInputs::from_installed(self, request)
                .map(std::sync::Arc::new);
        view
    }

    pub(crate) fn shared_candidate_inputs(
        &self,
    ) -> Option<std::sync::Arc<super::input_preparation::SharedCandidateInputs>> {
        self.shared_candidate_inputs.clone()
    }

    #[cfg(test)]
    pub(crate) const fn current_version_id(&self) -> crate::identity::data::VersionId {
        self.current_version_id
    }

    pub(crate) fn entity_aspect_plan(
        &self,
        kind_id: KindId,
    ) -> Option<&crate::schema::data::LoweredAspectContractPlan> {
        self.schema_contract_runtime
            .aspect_contract_plans
            .entity_plans
            .get(&kind_id)
    }

    pub(crate) const fn invariant_scale_inputs(&self) -> (usize, usize, usize, bool) {
        (
            self.entity_count,
            self.relation_count,
            self.version_depth,
            self.snapshot_pressure,
        )
    }
}
