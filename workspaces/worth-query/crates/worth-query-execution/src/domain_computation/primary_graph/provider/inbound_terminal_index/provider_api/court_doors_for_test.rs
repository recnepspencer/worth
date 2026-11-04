//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn lookup_completed_inbound_for_test(
        &self,
        correlation: &ExternalEffectCorrelationIdentity,
    ) -> Result<Option<WorthQueryCanonicalInboundCompletion>, WorthQueryInboundTerminalIndexDenial>
    {
        Self::lookup_completed_inbound(self, correlation)
    }
}
