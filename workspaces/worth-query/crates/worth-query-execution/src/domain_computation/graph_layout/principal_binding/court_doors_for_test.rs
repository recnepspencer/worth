//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::vocabulary::primary_graph::principal_binding::WorthQueryPrimaryPrincipalBindingLayout;

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn principal_bindings_for_test(
        &self,
    ) -> impl Iterator<Item = (&str, &WorthQueryPrimaryPrincipalBindingLayout)> {
        Self::principal_bindings(self)
    }
}

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn principal_binding_for_test(
        &self,
        name: &str,
    ) -> Option<&WorthQueryPrimaryPrincipalBindingLayout> {
        Self::principal_binding(self, name)
    }
}
