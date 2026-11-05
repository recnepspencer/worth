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

impl WorthQueryPrimaryPrincipalBindingLayout {
    #[doc(hidden)]
    pub fn into_parts_for_test(
        self,
    ) -> (
        KindId,
        AspectFieldLocator,
        AspectFieldLocator,
        KindId,
        KindId,
        AspectFieldLocator,
        DerivedIndexId,
    ) {
        (
            self.mapping_kind,
            self.identity_locator,
            self.status_locator,
            self.relation_kind,
            self.principal_kind,
            self.principal_identity_locator,
            self.index_id,
        )
    }
}
