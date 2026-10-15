//! Native selected-index currency, retained at its exact admitted basis.
use std::sync::atomic::{AtomicU8, Ordering};
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
/// A finite-use proof created only after existing selected read preparation/admission.
pub(in crate::domain_computation) struct PreparedSelectedReadIndexes {
    basis: AdmittedRelationalBranchBasis,
    remaining_security_checks: AtomicU8,
}
impl PreparedSelectedReadIndexes {
    pub(in crate::domain_computation::primary_graph) fn for_query(
        basis: AdmittedRelationalBranchBasis,
    ) -> Self {
        Self {
            basis,
            remaining_security_checks: AtomicU8::new(2),
        }
    }
    /// An actual capability access already owns complete selected-selector preparation.
    pub(in crate::domain_computation) fn for_capability<Schema, Capability, Operation, Input>(
        access: &crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<Schema,Capability,Operation,Input>,
    ) -> Option<Self>
    where
        Input:
            worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
                Schema,
                Capability,
            >,
    {
        Some(Self {
            basis: access.read_security_basis()?.clone(),
            remaining_security_checks: AtomicU8::new(1),
        })
    }
    pub(in crate::domain_computation) fn matches_basis(
        &self,
        actual: &AdmittedRelationalBranchBasis,
    ) -> bool {
        self.remaining_security_checks
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |remaining| {
                remaining.checked_sub(1)
            })
            .is_ok()
            && self.basis.admission_identity() == actual.admission_identity()
    }
}
