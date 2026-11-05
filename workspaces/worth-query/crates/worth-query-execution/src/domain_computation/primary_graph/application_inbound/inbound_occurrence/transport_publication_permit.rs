//! Installed transport completion admission to the shared World publication lane.

use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryInstalledTransportCompletionBinding, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::application_aftermath::{
    WorthQueryInboundCustody, WorthQueryTransportPublicationPermit,
    WorthQueryTransportPublicationPermitDenial,
};

pub(in crate::domain_computation) trait TransportPublicationPermitScopedOps<
    Schema: ApplicationSchema,
>:
    crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema>
{
    fn reserve_installed_transport_publication(
        &self,
        binding: &WorthQueryInstalledTransportCompletionBinding,
    ) -> Result<WorthQueryTransportPublicationPermit, WorthQueryTransportPublicationPermitDenial>
    {
        if binding.runtime() != self.authority_identity() {
            return Err(WorthQueryTransportPublicationPermitDenial::ForeignBinding);
        }
        WorthQueryInboundCustody::reserve_transport_publication(
            Arc::clone(&self.inbound().custody()),
            binding.operation(),
            binding.contract(),
        )
    }
}

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_inbound::host::ApplicationInboundHost<Schema> + ?Sized> TransportPublicationPermitScopedOps<Schema> for HostRuntime {}

mod court_doors_for_test;
pub use court_doors_for_test::TransportPublicationPermitDoorsOps;
