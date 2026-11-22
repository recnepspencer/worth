//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn binding_identity_for_test(&self, binding: TypeId) -> Option<OutputBindingIdentity> {
        Self::binding_identity(self, binding)
    }
}

impl WorthQueryApplicationOutputLineage {
    #[doc(hidden)]
    pub fn binding_type_for_test(&self, identity: &OutputBindingIdentity) -> TypeId {
        Self::binding_type(self, identity)
    }
}
