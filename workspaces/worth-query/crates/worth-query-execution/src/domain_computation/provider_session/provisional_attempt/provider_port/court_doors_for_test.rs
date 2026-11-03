//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProvisionalOverlayAdmission {
    #[doc(hidden)]
    pub fn new_for_test( session: crate::domain_computation::provider_session::WorthQueryProviderSessionView<'_>, binding_identity: &str, program_identity: &str, generation: u64) -> Self {
        Self::new(session, binding_identity, program_identity, generation)
    }
}
