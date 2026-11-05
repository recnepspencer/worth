//! Moved whole from `domain_computation::primary_graph::schema_layout::principal_binding`: the items below and every impl they own.
use crate::domain_computation::vocabulary::primary_graph::principal_binding::WorthQueryPrimaryPrincipalBindingLayout;
use worth_foundational::facade::AspectFieldLocator;

impl WorthQueryPrimaryPrincipalBindingLayout {
    #[doc(hidden)]
    pub fn identity_locator_for_test(&self) -> &AspectFieldLocator {
        &self.identity_locator
    }
}
