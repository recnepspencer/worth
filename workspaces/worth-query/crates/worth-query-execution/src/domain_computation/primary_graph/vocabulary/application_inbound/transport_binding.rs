//! Moved whole from `domain_computation::primary_graph::application_inbound::inbound_occurrence::transport_binding`: the items below and every impl they own.
use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity;
use worth_query_installation::facade::InstalledInboundOccurrenceContract;

/// Runtime-sealed operation and verifier audience for one installed outbox.
/// An external dispatch result cannot supply either field.
pub struct WorthQueryInstalledTransportCompletionBinding {
    operation: String,
    audience: String,
    contract: InstalledInboundOccurrenceContract,
    runtime: WorthQueryRuntimeAuthorityIdentity,
}

impl WorthQueryInstalledTransportCompletionBinding {
    pub(in crate::domain_computation) fn operation(&self) -> &str {
        &self.operation
    }

    pub(in crate::domain_computation) fn audience(&self) -> &str {
        &self.audience
    }

    pub(in crate::domain_computation) fn contract(&self) -> &InstalledInboundOccurrenceContract {
        &self.contract
    }

    pub(in crate::domain_computation) fn runtime(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime
    }
}

impl WorthQueryInstalledTransportCompletionBinding {
    #[doc(hidden)]
    pub fn runtime_for_test(&self) -> WorthQueryRuntimeAuthorityIdentity {
        Self::runtime(self)
    }
}
