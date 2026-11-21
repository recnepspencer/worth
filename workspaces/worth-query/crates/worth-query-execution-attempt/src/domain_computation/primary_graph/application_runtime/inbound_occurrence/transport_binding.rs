//! Exact installed route for completion observed on the outbound transport.

use worth_query_installation::facade::InstalledInboundOccurrenceContract;

use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;

/// Runtime-sealed operation and verifier audience for one installed outbox.
/// An external dispatch result cannot supply either field.
pub struct WorthQueryInstalledTransportCompletionBinding {
    pub operation: String,
    pub audience: String,
    pub contract: InstalledInboundOccurrenceContract,
    pub runtime: WorthQueryRuntimeAuthorityIdentity,
}

impl WorthQueryInstalledTransportCompletionBinding {
    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub fn audience(&self) -> &str {
        &self.audience
    }

    pub fn contract(&self) -> &InstalledInboundOccurrenceContract {
        &self.contract
    }

    pub fn runtime(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryTransportCompletionBindingDenial {
    OutboxWithoutInboundContract,
    OutboxContractMismatch,
    OutboxOperationUnbound,
    VerifierMissing,
    RecordedBindingUnavailable,
}


impl WorthQueryInstalledTransportCompletionBinding {
    #[doc(hidden)]
    pub fn runtime_for_test(&self) -> crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        self.runtime()
    }
}
