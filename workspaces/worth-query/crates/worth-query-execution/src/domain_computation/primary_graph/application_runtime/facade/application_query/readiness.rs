//! The runtime's consumer methods over `readiness` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::domain_computation::vocabulary::primary_graph::basis_registry::WorthQueryApplicationBasisIdentity;
use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenial;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema>
    super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub fn application_basis_token(
        &self,
        basis: &WorthQueryApplicationBasisIdentity,
    ) -> Result<String, WorthQueryApplicationQueryAdmissionDenial> {
        <Self as crate::domain_computation::primary_graph::application_query::ReadinessOps<
            Schema,
        >>::application_basis_token(self, basis)
    }

    pub fn application_commit_basis_token(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<String, WorthQueryApplicationQueryAdmissionDenial> {
        <Self as crate::domain_computation::primary_graph::application_query::ReadinessOps<
            Schema,
        >>::application_commit_basis_token(self, receipt)
    }
}
