//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationCommitDenial {
    #[doc(hidden)]
    pub fn with_provider_session_failure_for_test(
        self,
        failure: crate::domain_computation::WorthQueryProviderSessionFailure,
    ) -> Self {
        Self::with_provider_session_failure(self, failure)
    }
}
