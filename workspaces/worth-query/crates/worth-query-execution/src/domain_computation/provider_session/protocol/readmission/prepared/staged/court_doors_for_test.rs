//! Sealed test doors over this module's private members.
use super::*;

impl<'run> WorthQuerySessionBoundReadsAndEffects<'run> {
    #[doc(hidden)]
    pub fn execution_request_for_test(&self) -> worth_execution::ExecutionRequest<'run, 'run> {
        Self::execution_request(self)
    }
}

impl<'run> WorthQuerySessionBoundReadsAndEffects<'run> {
    #[doc(hidden)]
    pub fn provider_session_view_for_test(&self) -> WorthQueryProviderSessionView<'_> {
        Self::provider_session_view(self)
    }
}

impl<'run> WorthQuerySessionBoundReadsAndEffects<'run> {
    #[doc(hidden)]
    pub fn provisional_binding_identity_for_test(&self) -> &str {
        Self::provisional_binding_identity(self)
    }
}

impl<'run> WorthQuerySessionBoundReadsAndEffects<'run> {
    #[doc(hidden)]
    pub fn provisional_provider_for_test(&self) -> &WorthQueryGraphProviderAnchor {
        Self::provisional_provider(self)
    }
}

impl WorthQuerySessionReadAuthority<'_> {
    #[doc(hidden)]
    pub fn session_for_test(&self) -> WorthQueryProviderSessionView<'_> {
        Self::session(self)
    }
}
