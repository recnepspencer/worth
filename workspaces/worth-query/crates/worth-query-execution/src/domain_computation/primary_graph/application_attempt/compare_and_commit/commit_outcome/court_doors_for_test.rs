//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationUnresolvedCommitEvidence {
    #[doc(hidden)]
    pub fn from_provider_session_failure_for_test(
        recovery: WorthQueryApplicationCommitRecoveryKind,
        failure: &crate::domain_computation::provider_session::WorthQueryProviderSessionFailure,
    ) -> Self {
        Self::from_provider_session_failure(recovery, failure)
    }
}

impl WorthQueryApplicationUnresolvedCommitEvidence {
    #[doc(hidden)]
    pub fn with_cleanup_failure_for_test(self) -> Self {
        Self::with_cleanup_failure(self)
    }
}
