//! Native selected-index currency, retained at its exact admitted basis.
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
/// Exact-basis currency issued only by selected read preparation/admission.
pub(in crate::domain_computation) struct PreparedSelectedReadIndexes {
    basis: AdmittedRelationalBranchBasis,
}
impl PreparedSelectedReadIndexes {
    pub(in crate::domain_computation::primary_graph) fn for_query(
        basis: AdmittedRelationalBranchBasis,
    ) -> Self {
        Self { basis }
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
        })
    }
    pub(in crate::domain_computation) fn matches_basis(
        &self,
        actual: &AdmittedRelationalBranchBasis,
    ) -> bool {
        self.basis.admission_identity() == actual.admission_identity()
    }
}
