use super::WorthQueryWorkspace;

impl WorthQueryWorkspace {
    pub fn conditional_evaluation_resource_observation(
        &self,
    ) -> Option<super::super::WorthQueryConditionalEvaluationResourceObservation> {
        self.runtime.conditional_evaluation_resource_observation()
    }

    pub(crate) fn inject_classified_live_emission_failures(&mut self, count: usize) {
        self.runtime.inject_classified_live_emission_failures(count);
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

    pub(crate) fn admit_staged_conditional_owner_delivery<D: 'static, O: 'static, F: 'static>(
        &self,
        handle: &crate::ordinary::live::WorthQueryManagedLiveHandle,
        receipt: &worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
    ) -> Result<
        crate::runtime::WorthQueryAdmittedStagedOwnerDelivery,
        crate::runtime::WorthQueryStagedOwnerDeliveryAdmissionError,
    > {
        let target = self
            .resolve_live_artifact_target(handle.name())
            .map_err(|_| {
                crate::runtime::WorthQueryStagedOwnerDeliveryAdmissionError::missing_route()
            })?;
        self.runtime
            .admit_staged_conditional_owner_delivery::<D, O, F>(&target, receipt)
    }

    pub(crate) fn emit_classified_conditional_owner_delivery(
        &mut self,
        admitted: crate::runtime::WorthQueryAdmittedStagedOwnerDelivery,
        closure: &crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure,
        conditional: &crate::domain_installation::WorthQueryConditionalProvenance,
        impact: &crate::domain_installation::WorthQueryImpactDecision,
    ) -> Result<(), crate::runtime::WorthQueryClassifiedOwnerDeliveryEmissionError> {
        self.runtime.emit_classified_conditional_owner_delivery(
            admitted,
            closure,
            conditional,
            impact,
        )
    }

    pub(crate) fn register_installed_live_route<D: 'static, O: 'static, F: 'static>(
        &mut self,
        handle: &crate::ordinary::live::WorthQueryManagedLiveHandle,
        closure: &crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure,
    ) -> Result<(), crate::runtime::WorthQueryRuntimeError> {
        self.admit_managed_live_capability(handle.workspace_capability(), handle.name())?;
        let target = self.resolve_live_artifact_target(handle.name())?;
        self.runtime
            .register_installed_live_route::<D, O, F>(target, closure);
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

    pub(crate) fn reenter_retained_conditional_decision(
        &self,
        selected: &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
        request: worth_runtime_bridge::facade::BridgeConditionalDecisionReentryRequest<'_>,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence,
        (
            String,
            worth_runtime_bridge::facade::BridgeConditionalReentryCounters,
        ),
    > {
        self.runtime
            .reenter_retained_conditional_decision(selected, request)
    }
}
