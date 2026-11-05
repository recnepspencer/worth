//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_binding_identity::OutputBindingIdentity;


impl OutputBindingInventory {
    #[doc(hidden)]
    pub fn identity_for_test(&self, binding: TypeId) -> Option<OutputBindingIdentity> {
        Self::identity(self, binding)
    }
}

impl OutputBindingIdentity {
    #[doc(hidden)]
    pub fn as_str_for_test(&self) -> &str {
        Self::as_str(self)
    }
}
