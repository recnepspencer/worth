//! Moved whole from `domain_computation::primary_graph::schema_layout::principal_binding`: the items below and every impl they own.
use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::DerivedIndexId;
use crate::domain_computation::vocabulary::primary_graph::principal_binding::WorthQueryPrimaryPrincipalBindingLayout;



impl WorthQueryPrimaryPrincipalBindingLayout {
    #[doc(hidden)]
    pub fn identity_locator_for_test(&self) -> &AspectFieldLocator {
        &self.identity_locator
    }
}
