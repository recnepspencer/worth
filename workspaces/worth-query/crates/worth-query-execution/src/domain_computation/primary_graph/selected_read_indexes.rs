//! Native selected-index currency, retained at its exact admitted basis.
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
/// Exact-basis currency issued by selected preparation; retains no Native owner.
pub(in crate::domain_computation) struct PreparedSelectedReadIndexes {
    basis: worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
}
impl PreparedSelectedReadIndexes {
    pub(in crate::domain_computation::primary_graph) fn for_query(
        basis: AdmittedRelationalBranchBasis,
    ) -> Self {
        Self {
            basis: basis.admission_identity().clone(),
        }
    }
    /// Issuance carries the exact preparation identity, without the basis lease.
    pub(in crate::domain_computation) fn for_capability<Schema, Capability, Operation, Input>(
        access: &crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<Schema,Capability,Operation,Input>,
    ) -> Self
    where
        Input:
            worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
                Schema,
                Capability,
            >,
    {
        Self {
            basis: access.read_security_basis_identity().clone(),
        }
    }
    pub(in crate::domain_computation) fn matches_basis(
        &self,
        actual: &AdmittedRelationalBranchBasis,
    ) -> bool {
        &self.basis == actual.admission_identity()
    }
}
