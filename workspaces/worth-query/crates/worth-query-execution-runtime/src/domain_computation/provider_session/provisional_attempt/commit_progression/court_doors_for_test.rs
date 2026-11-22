//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryCommittedProviderSession {
    #[doc(hidden)]
    pub fn terminal_binding_for_test(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding
    {
        Self::terminal_binding(self)
    }
}
