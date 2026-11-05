//! Sealed test doors over this module's private members.
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn verify_completed_inbound_for_cleanup_for_test(
        &self,
        correlations: &[ExternalEffectCorrelationIdentity],
        now_unix_seconds: u64,
    ) -> Result<(), Denial> {
        Self::verify_completed_inbound_for_cleanup(self, correlations, now_unix_seconds)
    }
}
