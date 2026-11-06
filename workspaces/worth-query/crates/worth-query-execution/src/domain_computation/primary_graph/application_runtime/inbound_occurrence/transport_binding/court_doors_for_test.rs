//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn resolve_installed_transport_completion_binding_for_test(
        &self,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<
        WorthQueryInstalledTransportCompletionBinding,
        WorthQueryTransportCompletionBindingDenial,
    > {
        Self::resolve_installed_transport_completion_binding(self, owner)
    }
}

impl WorthQueryInstalledTransportCompletionBinding {
    #[doc(hidden)]
    pub fn runtime_for_test(&self) -> WorthQueryRuntimeAuthorityIdentity {
        Self::runtime(self)
    }
}
