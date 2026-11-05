//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_inbound::transport_binding::WorthQueryInstalledTransportCompletionBinding;

pub trait TransportPublicationPermitDoorsOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    #[doc(hidden)]
    fn reserve_installed_transport_publication_for_test(
        &self,
        binding: &WorthQueryInstalledTransportCompletionBinding,
    ) -> Result<WorthQueryTransportPublicationPermit, WorthQueryTransportPublicationPermitDenial>
    {
        Self::reserve_installed_transport_publication(self, binding)
    }
}

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> TransportPublicationPermitDoorsOps<Schema> for HostRuntime {}
