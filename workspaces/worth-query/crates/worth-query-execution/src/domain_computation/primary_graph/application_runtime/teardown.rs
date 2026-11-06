//! Ordered destruction of concrete custody interleaved with typed registries.
use super::WorthQueryApplicationRuntimeCore;
use crate::domain_computation::primary_graph::{
    application_contribution, application_output_demand, provider::WorthQueryPrimaryGraphProvider,
    workflow,
};

// These owners followed the typed conditional registry before the core split.
// Move exclusive custody and retain shared backing before cleanup can unwind.
#[derive(Default)]
pub(super) struct ApplicationPostConditionalTeardown {
    provider: Option<std::sync::Arc<WorthQueryPrimaryGraphProvider>>,
    authority: Option<super::home_retirement::PrimaryGraphParticipationCustody>,
    meanings: Option<super::super::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry>,
    transport: Option<std::sync::Arc<dyn crate::domain_computation::application_aftermath::WorthQueryExternalEffectTransport>>,
    verifiers: std::collections::BTreeMap<String, std::sync::Arc<super::inbound_occurrence::WorthQueryInstalledInboundVerifier>>,
    inbound: Option<std::sync::Arc<std::sync::Mutex<crate::domain_computation::application_aftermath::WorthQueryInboundCustody>>>,
    completions: super::inbound_publication::InstalledTransportCompletionCustody,
    recovery: Option<std::sync::Arc<crate::domain_computation::managed_run::WorthQueryRecoveryHandleRegistry>>,
}
impl ApplicationPostConditionalTeardown {
    pub(super) fn take_from(&mut self, core: &mut WorthQueryApplicationRuntimeCore) {
        self.provider = core.primary_provider.retained_for_teardown();
        self.authority = core.primary_graph_authority.take();
        self.meanings = core.source_meanings.take();
        self.transport = core.external_effect_transport.take();
        self.verifiers = std::mem::take(
            core.inbound_verifiers
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        self.inbound = Some(std::sync::Arc::clone(&core.inbound_custody));
        self.completions = std::mem::take(
            core.transport_completion_custody
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        self.recovery = Some(std::sync::Arc::clone(&core.recovery_handles));
    }
}
impl Drop for ApplicationPostConditionalTeardown {
    fn drop(&mut self) {
        drop(self.provider.take());
        drop(self.authority.take());
        drop(self.meanings.take());
        drop(self.transport.take());
        drop(std::mem::take(&mut self.verifiers));
        drop(self.inbound.take());
        drop(std::mem::take(&mut self.completions));
        drop(self.recovery.take());
    }
}

#[derive(Default)]
pub(super) struct ApplicationOutputTeardown {
    producers: Option<application_contribution::WorthQueryInstalledOutputProducerRoutes>,
    readiness: Option<application_contribution::WorthQueryInstalledOutputReadinessRoutes>,
    demands: Option<application_output_demand::WorthQueryOutputDemandRegistry>,
    recovered: Option<application_output_demand::WorthQueryRecoveredOutputs>,
}

impl ApplicationOutputTeardown {
    pub(super) fn take_from(&mut self, core: &mut WorthQueryApplicationRuntimeCore) {
        self.producers = Some(std::mem::take(&mut core.output_producer_routes));
        self.readiness = Some(std::mem::take(&mut core.output_readiness_routes));
        // A registry Default allocates. Retain the same Arc instead: the core's
        // earlier decrement cannot destroy this backing before its old slot.
        self.demands = Some(core.output_demands.clone());
        self.recovered = Some(std::mem::take(&mut core.recovered_outputs));
    }
}

impl Drop for ApplicationOutputTeardown {
    fn drop(&mut self) {
        drop(self.producers.take());
        drop(self.readiness.take());
        drop(self.demands.take());
        drop(self.recovered.take());
    }
}

#[derive(Default)]
pub(super) struct ApplicationWorkflowTeardown {
    coverage: Option<workflow::adoption::WorkflowVocabularyCoverageRegistry>,
}

impl ApplicationWorkflowTeardown {
    pub(super) fn take_from(&mut self, core: &mut WorthQueryApplicationRuntimeCore) {
        self.coverage = Some(std::mem::take(&mut core.workflow_coverage));
    }
}

impl Drop for ApplicationWorkflowTeardown {
    fn drop(&mut self) {
        drop(self.coverage.take());
    }
}
