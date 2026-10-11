//! Native inspection binds a Current certificate to the actual row it read.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::CurrentBasis;
impl RequiredWorkMembership {
    pub(in crate::domain_computation::primary_graph) fn certify_current_basis(
        self: &Arc<Self>,
        basis: CurrentBasis,
    ) -> Arc<Self> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .current_basis = Some(basis);
        Arc::clone(self)
    }
    pub(in crate::domain_computation::primary_graph) fn certified_at(
        &self,
        basis: &CurrentBasis,
    ) -> bool {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .current_basis
            .as_ref()
            == Some(basis)
    }
}
impl SelectedReadyReadmission {
    pub(in crate::domain_computation::primary_graph) fn certify_current_record(
        &self,
        basis: CurrentBasis,
    ) -> Arc<RequiredWorkMembership> {
        self.membership.certify_current_basis(basis)
    }
}
