use crate::identity::data::KindId;
use crate::runtime::state::subsystems::schema_contract_runtime::SchemaContractRuntimeSubsystem;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;

/// The configuration and immutable read handles selected by the owner.
pub(crate) struct InvariantRuntimeInputs<'a> {
    pub(crate) selected_registry: crate::schema::SchemaRegistryCustody,
    pub(crate) prior_registry: Option<crate::schema::SchemaRegistryCustody>,
    pub(crate) commit_work_budget: Option<crate::execution::RequestWorkBudget>,
    pub(crate) config: std::sync::Arc<crate::config::data::RelationalRuntimeConfig>,
    pub(crate) schema_contract_runtime: std::sync::Arc<SchemaContractRuntimeSubsystem>,
    pub(crate) instrumentation: &'a RuntimeInstrumentation,
    #[cfg(test)]
    pub(crate) current_version_id: crate::identity::data::VersionId,
    pub(crate) entity_count: usize,
    pub(crate) relation_count: usize,
    pub(crate) version_depth: usize,
    pub(crate) snapshot_pressure: bool,
}

/// Read-only validation projection shared by the runtime and preparation port.
///
/// The configuration and the contract runtime lowered from it are held as the
/// one snapshot taken when the view was made, so every question this view
/// answers is answered against the same configuration even if the owner
/// installs a new one meanwhile.
#[derive(Clone)]
pub(crate) struct InvariantRuntimeView<'a> {
    pub(crate) selected_registry: crate::schema::SchemaRegistryCustody,
    pub(crate) prior_registry: Option<crate::schema::SchemaRegistryCustody>,
    pub(crate) custom_kernel_posture: worth_foundational::ExecutionPosture,
    pub(crate) commit_work_budget: Option<crate::execution::RequestWorkBudget>,
    pub(crate) config: std::sync::Arc<crate::config::data::RelationalRuntimeConfig>,
    pub(crate) schema_contract_runtime: std::sync::Arc<SchemaContractRuntimeSubsystem>,
    instrumentation: &'a RuntimeInstrumentation,
    counter_sink: Option<
        &'a std::sync::Mutex<
            crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
        >,
    >,
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
    pub(crate) fn from_inputs(inputs: InvariantRuntimeInputs<'a>) -> Self {
        Self {
            selected_registry: inputs.selected_registry,
            prior_registry: inputs.prior_registry,
            custom_kernel_posture: worth_foundational::ExecutionPosture::Serial,
            commit_work_budget: inputs.commit_work_budget,
            config: inputs.config,
            schema_contract_runtime: inputs.schema_contract_runtime,
            instrumentation: inputs.instrumentation,
            counter_sink: None,
            shared_candidate_inputs: None,
            #[cfg(test)]
            current_version_id: inputs.current_version_id,
            entity_count: inputs.entity_count,
            relation_count: inputs.relation_count,
            version_depth: inputs.version_depth,
            snapshot_pressure: inputs.snapshot_pressure,
        }
    }

    pub(crate) fn rejudges_home_target(&self) -> bool {
        self.prior_registry.is_some()
    }

    pub(crate) fn performance_access(&self) -> super::metrics::InvariantMetrics<'a> {
        super::metrics::InvariantMetrics::new(
            self.counter_sink
                .unwrap_or(&self.instrumentation.complexity_counters),
        )
    }

    pub(crate) fn with_counter_sink<'round>(
        &self,
        counters: &'round std::sync::Mutex<
            crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
        >,
    ) -> InvariantRuntimeView<'round>
    where
        'a: 'round,
    {
        let mut view: InvariantRuntimeView<'round> = self.clone();
        view.counter_sink = Some(counters);
        view.shared_candidate_inputs = None;
        view
    }

    pub(crate) fn with_candidate_input_plan<'state>(
        &self,
        request: &super::InvariantExecutionRequest<'state>,
    ) -> InvariantRuntimeView<'state>
    where
        'a: 'state,
    {
        InvariantRuntimeView {
            selected_registry: self.selected_registry.clone(),
            prior_registry: self.prior_registry.clone(),
            custom_kernel_posture: self.custom_kernel_posture,
            commit_work_budget: self.commit_work_budget.clone(),
            config: std::sync::Arc::clone(&self.config),
            schema_contract_runtime: std::sync::Arc::clone(&self.schema_contract_runtime),
            instrumentation: self.instrumentation,
            counter_sink: self.counter_sink,
            shared_candidate_inputs:
                super::input_preparation::SharedCandidateInputs::from_installed(self, request)
                    .map(std::sync::Arc::new),
            #[cfg(test)]
            current_version_id: self.current_version_id,
            entity_count: self.entity_count,
            relation_count: self.relation_count,
            version_depth: self.version_depth,
            snapshot_pressure: self.snapshot_pressure,
        }
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
