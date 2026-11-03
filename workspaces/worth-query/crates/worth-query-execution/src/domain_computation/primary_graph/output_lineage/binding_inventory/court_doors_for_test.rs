//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn binding_identity_for_test(&self, binding: TypeId) -> Option<OutputBindingIdentity> {
        Self::binding_identity(self, binding)
    }
}
