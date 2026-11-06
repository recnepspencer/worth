//! Sealed test doors over this module's private members.
use crate::domain_computation::vocabulary::primary_graph::commit_authority_binding::WorthQueryRetainedGovernedInput;

impl WorthQueryRetainedGovernedInput {
    #[doc(hidden)]
    pub fn governed_identity_for_test(&self) -> Option<[u8; 32]> {
        Self::governed_identity(self)
    }
}
