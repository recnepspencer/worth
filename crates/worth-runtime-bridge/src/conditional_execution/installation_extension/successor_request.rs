use super::super::{
    BridgeConditionalProviderSemantics, BridgeConditionalWakeProvider,
    BridgeInstalledConditionalLowering, BridgeOwnedConditionalInstallationRequest,
};

impl BridgeInstalledConditionalLowering {
    /// Retain the installed operation, correspondence, and non-wake provider
    /// meaning while replacing the host-owned wake predicate for a successor.
    pub fn successor_with_wake_provider<Provider>(
        &self,
        provider: Provider,
    ) -> BridgeOwnedConditionalInstallationRequest
    where
        Provider: BridgeConditionalWakeProvider + BridgeConditionalProviderSemantics,
    {
        BridgeOwnedConditionalInstallationRequest {
            contract: self.contract.clone(),
            location: self.location.clone(),
            dependencies: self.semantic_dependencies().cloned().collect(),
            providers: self.providers.clone().wake(provider),
        }
    }
}
