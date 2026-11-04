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

impl WorthQueryPrimaryPrincipalBindingLayout {
    #[doc(hidden)]
    pub fn identity_locator_for_test(&self) -> &AspectFieldLocator {
        &self.identity_locator
    }
}

impl WorthQueryPrimaryPrincipalBindingLayout {
    #[doc(hidden)]
    pub fn index_id_for_test(&self) -> &DerivedIndexId {
        &self.index_id
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

impl WorthQueryPrimaryPrincipalBindingLayout {
    #[doc(hidden)]
    pub fn mapping_kind_for_test(&self) -> &KindId {
        &self.mapping_kind
    }
}

impl WorthQueryPrimaryPrincipalBindingLayout {
    #[doc(hidden)]
    pub fn principal_identity_locator_for_test(&self) -> &AspectFieldLocator {
        &self.principal_identity_locator
    }
}

impl WorthQueryPrimaryPrincipalBindingLayout {
    #[doc(hidden)]
    pub fn principal_kind_for_test(&self) -> &KindId {
        &self.principal_kind
    }
}

impl WorthQueryPrimaryPrincipalBindingLayout {
    #[doc(hidden)]
    pub fn relation_kind_for_test(&self) -> &KindId {
        &self.relation_kind
    }
}

impl WorthQueryPrimaryPrincipalBindingLayout {
    #[doc(hidden)]
    pub fn status_locator_for_test(&self) -> &AspectFieldLocator {
        &self.status_locator
    }
}

impl WorthQueryPrimaryPrincipalBindingLayout {
    #[doc(hidden)]
    pub fn principal_identity_locator_into_for_test(self) -> AspectFieldLocator {
        self.principal_identity_locator
    }
}

impl WorthQueryPrimaryPrincipalBindingLayout {
    #[doc(hidden)]
    pub fn status_locator_into_for_test(self) -> AspectFieldLocator {
        self.status_locator
    }
}
