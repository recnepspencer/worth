//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryPrimaryGraphLayout {
    #[doc(hidden)]
    pub fn principal_bindings_for_test(
        &self,
    ) -> impl Iterator<Item = (&str, &WorthQueryPrimaryPrincipalBindingLayout)> {
        Self::principal_bindings(self)
    }
}
