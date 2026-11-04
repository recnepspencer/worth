//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryCanonicalInboundCompletion {
    #[doc(hidden)]
    pub fn matches_transport_observation_for_test(
        &self,
        evidence: &InstalledTransportCompletion,
        binding: &WorthQueryInstalledTransportCompletionBinding,
    ) -> bool {
        Self::matches_transport_observation(self, evidence, binding)
    }
}
