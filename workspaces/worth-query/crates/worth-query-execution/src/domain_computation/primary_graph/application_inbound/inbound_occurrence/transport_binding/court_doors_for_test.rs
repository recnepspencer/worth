//! Sealed test doors over this module's private members.
use super::*;

pub trait TransportBindingDoorsOps<Schema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    fn resolve_installed_transport_completion_binding_for_test(
        &self,
        owner: &WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<
        WorthQueryInstalledTransportCompletionBinding,
        WorthQueryTransportCompletionBindingDenial,
    > {
        Self::resolve_installed_transport_completion_binding(self, owner)
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> TransportBindingDoorsOps<Schema> for HostRuntime where
    Schema: ApplicationSchema, {}

impl WorthQueryInstalledTransportCompletionBinding {
    #[doc(hidden)]
    pub fn runtime_for_test(&self) -> WorthQueryRuntimeAuthorityIdentity {
        Self::runtime(self)
    }
}
