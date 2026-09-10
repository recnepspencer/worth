use super::WorthQueryWorkspace;

impl WorthQueryWorkspace {
    pub fn conditional_evaluation_resource_observation(
        &self,
    ) -> Option<super::super::WorthQueryConditionalEvaluationResourceObservation> {
        self.runtime.conditional_evaluation_resource_observation()
    }

    pub(crate) fn replace_conditional_lowerings_for_test_from<
        D: 'static,
        O: 'static,
        F: 'static,
    >(
        &mut self,
        donor: &Self,
    ) -> Result<(), &'static str> {
        let donor_nodes = donor.runtime.conditional_nodes::<D, O, F>();
        let current_domain = self
            .runtime
            .domain_installation_registry
            .domain::<D>()
            .map_err(|_| "recipient domain is not installed")?;
        self.runtime
            .conditional_execution_registry
            .replace_lowerings_for_test::<D, O, F>(
                &donor_nodes,
                current_domain.authority().runtime_authority().as_u64(),
                current_domain.installation_generation().ordinal(),
            )
    }

    pub(crate) fn register_installed_live_route(
        &mut self,
        handle: &crate::ordinary::live::WorthQueryManagedLiveHandle,
        closure: &crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure,
    ) -> Result<(), crate::runtime::WorthQueryRuntimeError> {
        self.admit_managed_live_capability(handle.workspace_capability(), handle.name())?;
        let target = self.resolve_live_artifact_target(handle.name())?;
        self.runtime.register_installed_live_route(target, closure);
        Ok(())
    }

    pub(crate) fn execute_installed_conditional(
        &self,
        selected: &std::sync::Arc<
            worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
        >,
        request: worth_runtime_bridge::facade::BridgeConditionalExecutionRequest<'_>,
        context: &mut dyn std::any::Any,
    ) -> Result<
        super::super::WorthQueryExecutedConditional,
        (
            worth_runtime_bridge::facade::BridgeConditionalDenialKind,
            String,
            worth_signal::facade::SignalConditionalDecisionCounters,
            usize,
        ),
    > {
        self.runtime.execute_conditional(selected, request, context)
    }
}
