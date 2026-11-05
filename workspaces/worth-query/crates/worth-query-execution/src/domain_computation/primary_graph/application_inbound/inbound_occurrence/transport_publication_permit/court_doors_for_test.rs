//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn reserve_installed_transport_publication_for_test(
        &self,
        binding: &WorthQueryInstalledTransportCompletionBinding,
    ) -> Result<WorthQueryTransportPublicationPermit, WorthQueryTransportPublicationPermitDenial>
    {
        Self::reserve_installed_transport_publication(self, binding)
    }
}
