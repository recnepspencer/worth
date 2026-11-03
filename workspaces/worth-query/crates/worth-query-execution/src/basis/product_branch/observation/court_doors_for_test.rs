//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryProductBranchLease {
    #[doc(hidden)]
    pub fn observation_for_test(&self) -> &worth_runtime_world::facade::ProductBranchObservation {
        Self::observation(self)
    }
}

impl WorthQueryProductBranchLease {
    #[doc(hidden)]
    pub fn publication_binding_for_test(
        &self) -> crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding {
        Self::publication_binding(self)
    }
}

impl WorthQueryProductBranchLease {
    #[doc(hidden)]
    pub fn relational_basis_for_test(
        &self) -> &worth_relational::facade::branch::AdmittedRelationalBranchBasis {
        Self::relational_basis(self)
    }
}
