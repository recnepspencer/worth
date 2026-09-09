use crate::domain_installation::{
    WorthQueryDomainExecutionIndexRebuildReport, WorthQueryDomainHandleDenial,
    WorthQueryDomainInstallationLookupCounters, WorthQueryDomainInstallationReceipt,
    WorthQueryDomainRebindDenial, WorthQueryDomainRebindReceipt, WorthQueryDomainRebindRequest,
    WorthQueryInstalledDomainAuthorityWitness, WorthQueryInstalledDomainHandle,
    WorthQueryInstalledDomainOperation, WorthQueryInstalledDomainOperationLookupDenial,
    WorthQueryReboundDomainHandle,
};

use super::WorthQueryRuntime;

impl WorthQueryRuntime {
    pub(crate) fn query_execution_runtime(
        &self,
    ) -> &worth_query_execution::facade::runtime::WorthQueryExecutionRuntime {
        &self.execution_runtime
    }

    pub(crate) fn query_execution_installation_authority(
        &self,
    ) -> &worth_query_execution::facade::runtime::WorthQueryExecutionInstallationAuthority {
        &self.execution_installation_authority
    }

    pub(crate) fn workflow_parallel_admission_provider<D: 'static, O: 'static, F: 'static>(
        &self,
    ) -> Option<
        std::sync::Arc<
            crate::domain_installation::WorthQueryInstalledWorkflowParallelAdmissionProvider,
        >,
    > {
        self.workflow_parallel_admission_provider_registry
            .get::<D, O, F>()
    }

    pub(crate) fn workflow_stage_executor<D: 'static, O: 'static, F: 'static>(
        &self,
    ) -> Option<std::sync::Arc<crate::domain_installation::WorthQueryInstalledWorkflowStageExecutor>>
    {
        self.workflow_stage_executor_registry.get::<D, O, F>()
    }

    pub(crate) fn domain_operation_executor<D: 'static, O: 'static, F: 'static>(
        &self,
    ) -> Option<
        std::sync::Arc<crate::domain_installation::WorthQueryInstalledDomainOperationExecutor>,
    > {
        self.domain_operation_executor_registry.get::<D, O, F>()
    }

    pub(crate) fn consumer_support_profile(
        &self,
    ) -> &crate::domain_installation::WorthQueryConsumerSupportProfile {
        &self.consumer_support_profile
    }

    pub fn graph_participation<G: 'static>(
        &self,
        _marker: G,
    ) -> Result<
        crate::domain_installation::WorthQueryInstalledGraphParticipation<G>,
        crate::domain_installation::WorthQueryGraphParticipationLookupDenial,
    > {
        self.graph_participation_registry
            .get::<G>()
            .map(crate::domain_installation::WorthQueryInstalledGraphParticipation::new)
    }

    pub fn operation<D, O, F>(
        &self,
        handle: &WorthQueryInstalledDomainHandle<D>,
        _operation: O,
        _family: F,
    ) -> Result<
        WorthQueryInstalledDomainOperation<D, O, F>,
        WorthQueryInstalledDomainOperationLookupDenial,
    >
    where
        D: 'static,
        O: 'static,
        F: 'static,
    {
        self.resolve_installed_operation::<D, O, F>(handle)
    }

    pub(crate) fn resolve_installed_operation<D: 'static, O: 'static, F: 'static>(
        &self,
        handle: &WorthQueryInstalledDomainHandle<D>,
    ) -> Result<
        WorthQueryInstalledDomainOperation<D, O, F>,
        WorthQueryInstalledDomainOperationLookupDenial,
    > {
        self.validate_installed_domain_handle(handle)
            .map_err(WorthQueryInstalledDomainOperationLookupDenial::domain)?;
        let domain_marker = std::any::TypeId::of::<D>();
        let operation_marker = std::any::TypeId::of::<O>();
        let family_marker = std::any::TypeId::of::<F>();
        let resolved = self
            .installed_domain_execution_index()
            .domain_operation_authority(domain_marker, operation_marker, family_marker)
            .ok_or_else(WorthQueryInstalledDomainOperationLookupDenial::operation_not_installed)?;
        let graph_bindings = self
            .installed_domain_execution_index()
            .domain_operation_graph_bindings(domain_marker, operation_marker, family_marker)
            .to_vec();
        let graph_bindings_retained = graph_bindings.len();
        Ok(WorthQueryInstalledDomainOperation::mint(
            handle.authority_arc(),
            resolved.authority,
            resolved.workflow_graph,
            graph_bindings,
            crate::domain_installation::WorthQueryInstalledDomainOperationLookupCounters {
                authority_checks: 1,
                indexed_operation_lookups: 1,
                graph_binding_lookups: 1,
                graph_bindings_retained,
                ..Default::default()
            },
        ))
    }

    pub(crate) fn installed_graph_participation(
        &self,
        marker: std::any::TypeId,
    ) -> Result<
        std::sync::Arc<crate::domain_installation::WorthQueryInstalledGraphParticipationRecord>,
        crate::domain_installation::WorthQueryGraphParticipationLookupDenial,
    > {
        self.graph_participation_registry.get_by_marker(marker)
    }

    pub fn domain<D: 'static>(
        &self,
        _marker: D,
    ) -> Result<WorthQueryInstalledDomainHandle<D>, WorthQueryDomainHandleDenial> {
        self.domain_installation_registry.domain::<D>()
    }

    pub fn domain_installation_receipt<D: 'static>(
        &self,
        _marker: D,
    ) -> Option<&WorthQueryDomainInstallationReceipt> {
        self.domain_installation_registry.receipt::<D>()
    }

    pub fn domain_installation_receipts(
        &self,
    ) -> impl ExactSizeIterator<Item = &WorthQueryDomainInstallationReceipt> {
        self.domain_installation_registry.receipts()
    }

    pub fn validate_installed_domain_handle<D: 'static>(
        &self,
        handle: &WorthQueryInstalledDomainHandle<D>,
    ) -> Result<(), WorthQueryDomainHandleDenial> {
        self.domain_installation_registry.validate(handle)
    }

    pub(crate) fn validate_installed_domain_witness<D: 'static>(
        &self,
        witness: &WorthQueryInstalledDomainAuthorityWitness,
    ) -> Result<(), WorthQueryDomainHandleDenial> {
        self.domain_installation_registry
            .validate_authority::<D>(witness.authority())
    }

    pub(crate) fn validate_installed_domain_authority(
        &self,
        witness: &WorthQueryInstalledDomainAuthorityWitness,
    ) -> Result<(), WorthQueryDomainHandleDenial> {
        self.domain_installation_registry
            .validate_erased_authority(witness.authority())
    }

    pub fn domain_installation_lookup_counters(
        &self,
    ) -> WorthQueryDomainInstallationLookupCounters {
        self.domain_installation_registry.lookup_counters()
    }

    pub fn verify_domain_execution_index_rebuild(
        &self,
    ) -> WorthQueryDomainExecutionIndexRebuildReport {
        self.domain_installation_registry
            .rebuild_execution_index_report()
    }

    pub fn rebind_domain<D: 'static>(
        &self,
        request: WorthQueryDomainRebindRequest<D>,
    ) -> Result<WorthQueryReboundDomainHandle<D>, WorthQueryDomainRebindDenial> {
        let prior = request.into_prior();
        let current = self
            .domain_installation_registry
            .domain::<D>()
            .map_err(|_| WorthQueryDomainRebindDenial::domain_not_installed(&prior))?;
        if prior.package_identity() != current.package_identity() {
            return Err(WorthQueryDomainRebindDenial::package_meaning_changed(
                &prior,
                current.authority(),
            ));
        }
        let current_witness = current.authority_witness();
        let receipt = WorthQueryDomainRebindReceipt::new(&prior, &current_witness);
        Ok(WorthQueryReboundDomainHandle::new(current, receipt))
    }

    pub(crate) fn installed_domain_execution_index(
        &self,
    ) -> &crate::domain_installation::WorthQueryInstalledDomainExecutionIndex {
        self.domain_installation_registry.execution_index()
    }

    pub(crate) fn installed_domain_authority_by_marker(
        &self,
        marker: std::any::TypeId,
    ) -> Option<std::sync::Arc<crate::domain_installation::WorthQueryInstalledDomainAuthority>>
    {
        self.domain_installation_registry
            .authority_by_marker(marker)
    }

    #[cfg(test)]
    pub(crate) fn destroy_and_rebuild_domain_execution_index(
        &mut self,
    ) -> WorthQueryDomainExecutionIndexRebuildReport {
        let report = self
            .domain_installation_registry
            .destroy_and_rebuild_execution_index();
        self.execution_runtime
            .replace_rebuilt_installation(self.domain_installation_registry.retain_portable_index())
            .expect("a deterministic installed-index rebuild must preserve execution identity");
        report
    }

    pub(crate) fn replace_domain_installation_with_successor_generation(
        &mut self,
    ) -> Result<(), crate::runtime::WorthQueryRuntimeError> {
        let successor = self
            .domain_installation_registry
            .prepare_runtime_reconstitution();
        if let Some(product) = self.installed_product.as_mut() {
            let selected = product
                .world
                .admit_product_branch(product.world.default_branch())
                .map_err(|denial| {
                    crate::runtime::WorthQueryRuntimeError::InvariantRegistration {
                        stage: "conditional_product_reconstitution_admission",
                        message: format!("{denial:?}"),
                    }
                })?;
            let current = &mut product.conditional;
            let (candidate, registry) =
                crate::runtime::WorthQueryRuntimeBuilder::prepare_conditional_reconstitution(
                    current,
                    &selected,
                    &self.conditional_execution_registry,
                    &successor,
                )?;
            current
                .activate_conditional_reconstitution(candidate)
                .map_err(|denial| {
                    crate::runtime::WorthQueryRuntimeError::InvariantRegistration {
                        stage: "conditional_runtime_reconstitution",
                        message: format!("{:?}: {}", denial.kind(), denial.detail()),
                    }
                })?;
            self.conditional_execution_registry = registry;
        }
        self.domain_installation_registry
            .commit_runtime_reconstitution(successor);
        Ok(())
    }
}
