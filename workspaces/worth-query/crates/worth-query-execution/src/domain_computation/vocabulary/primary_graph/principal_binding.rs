//! Moved whole from `domain_computation::primary_graph::schema_layout::principal_binding::court_doors_for_test`: the items below and every impl they own.
use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::indexes::DerivedIndexId;

impl WorthQueryPrimaryPrincipalBindingLayout {
    #[doc(hidden)]
    pub fn index_id_for_test(&self) -> &DerivedIndexId {
        &self.index_id
    }
}

#[derive(Clone, Debug)]
pub struct WorthQueryPrimaryPrincipalBindingLayout {
    pub(in crate::domain_computation) mapping_kind: KindId,
    pub(in crate::domain_computation) identity_locator: AspectFieldLocator,
    pub(in crate::domain_computation) status_locator: AspectFieldLocator,
    pub(in crate::domain_computation) relation_kind: KindId,
    pub(in crate::domain_computation) principal_kind: KindId,
    pub(in crate::domain_computation) principal_identity_locator: AspectFieldLocator,
    pub(in crate::domain_computation) index_id: DerivedIndexId,
}

impl WorthQueryPrimaryPrincipalBindingLayout {
    pub(in crate::domain_computation) const fn selector_index(&self) -> DerivedIndexId {
        self.index_id
    }
}
