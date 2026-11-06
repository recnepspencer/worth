//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationCommitDeferred {
    #[doc(hidden)]
    pub fn from_provider_session_for_test(
        deferred: crate::domain_computation::provider_session::WorthQueryProviderSessionCommitDeferred,
    ) -> Self {
        Self::from_provider_session(deferred)
    }
}
