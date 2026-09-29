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

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation) fn reserve_installed_transport_publication(
        &self,
        binding: &WorthQueryInstalledTransportCompletionBinding,
    ) -> Result<WorthQueryTransportPublicationPermit, WorthQueryTransportPublicationPermitDenial>
    {
        if binding.runtime() != self.runtime.authority_identity() {
            return Err(WorthQueryTransportPublicationPermitDenial::ForeignBinding);
        }
        WorthQueryInboundCustody::reserve_transport_publication(
            Arc::clone(&self.inbound_custody),
            binding.operation(),
            binding.contract(),
        )
    }
}
