use super::super::{
    BridgeConditionalProviderSemantics, BridgeConditionalWakeProvider,
    BridgeInstalledConditionalLowering, BridgeOwnedConditionalInstallationRequest,
};

impl BridgeInstalledConditionalLowering {
    /// Replace the condition evaluated by a runtime-predicate definition while
    /// retaining the operation, dependency correspondence and other providers.
    pub fn successor_with_authored_condition_provider<P>(
        &self,
        provider: P,
    ) -> BridgeOwnedConditionalInstallationRequest
    where
        P: super::super::BridgeConditionalConditionProvider
            + super::super::BridgeAuthoredConditionalProvider,
    {
        BridgeOwnedConditionalInstallationRequest {
            contract: self.contract.as_ref().clone(),
            location: self.location.as_ref().clone(),
            dependencies: self.semantic_dependencies().cloned().collect(),
            providers: self.providers.clone().authored_condition(provider),
        }
    }

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
            contract: self.contract.as_ref().clone(),
            location: self.location.as_ref().clone(),
            dependencies: self.semantic_dependencies().cloned().collect(),
            providers: self.providers.clone().wake(provider),
        }
    }
    /// Authored successors declare complete provider parameters; the durable
    /// owner reconstructs them through its admitted fresh decoder set.
    pub fn successor_with_authored_wake_provider<P>(
        &self,
        provider: P,
    ) -> BridgeOwnedConditionalInstallationRequest
    where
        P: BridgeConditionalWakeProvider + super::super::BridgeAuthoredConditionalProvider,
    {
        BridgeOwnedConditionalInstallationRequest {
            contract: self.contract.as_ref().clone(),
            location: self.location.as_ref().clone(),
            dependencies: self.semantic_dependencies().cloned().collect(),
            providers: self.providers.clone().authored_wake(provider),
        }
    }
}
