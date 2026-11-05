//! Ordered destruction of concrete custody interleaved with typed registries.
use super::WorthQueryApplicationRuntimeCore;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;

// These owners followed the typed conditional registry before the core split.
// Move exclusive custody and retain shared backing before cleanup can unwind.
#[derive(Default)]
pub(super) struct ApplicationPostConditionalTeardown {
    provider: Option<std::sync::Arc<WorthQueryPrimaryGraphProvider>>,
    authority: Option<worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority>,
    meanings: Option<super::super::application_query::observed_source::source_identity::WorthQueryObservedSourceMeaningRegistry>,
    transport: Option<std::sync::Arc<dyn crate::domain_computation::application_aftermath::WorthQueryExternalEffectTransport>>,
    verifiers: std::collections::BTreeMap<String, std::sync::Arc<crate::domain_computation::primary_graph::application_inbound::inbound_occurrence::WorthQueryInstalledInboundVerifier>>,
    inbound: Option<std::sync::Arc<std::sync::Mutex<crate::domain_computation::application_aftermath::WorthQueryInboundCustody>>>,
    completions: crate::domain_computation::primary_graph::application_inbound::inbound_publication::InstalledTransportCompletionCustody,
    recovery: Option<std::sync::Arc<crate::domain_computation::managed_run::WorthQueryRecoveryHandleRegistry>>,
}
impl ApplicationPostConditionalTeardown {
    pub(super) fn take_from(&mut self, core: &mut WorthQueryApplicationRuntimeCore) {
        self.provider = Some(std::sync::Arc::clone(&core.primary_provider));
        self.authority = core.primary_graph_authority.take();
        self.meanings = core.source_meanings.take();
        self.transport = core.external_effect_transport.take();
        self.verifiers = core.inbound.take_verifiers();
        self.inbound = Some(std::sync::Arc::clone(core.inbound.custody()));
        self.completions = core.inbound.take_completions();
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
pub(super) struct ApplicationWorkflowTeardown {
    coverage: Option<crate::domain_computation::primary_graph::application_attempt::workflow::adoption::WorkflowVocabularyCoverageRegistry>,
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
