//! Authenticated deployment issuer revocation through the current program owner.
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};
use bank_domain::schema::{
    ProgramDeploymentRevocationChanged, ProgramDeploymentRevocationDenial,
    ProgramDeploymentRevocationInput, ProgramDeploymentRevocationIntent,
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
    },
    runtime::ExecutionAllocationPolicy,
};
impl BankIdentityRuntime {
    /// Revokes one explicit deployment grant. Query's recorded issuer relation
    /// authorizes the actor; authentication or an employee role alone does not.
    pub fn revoke_program_deployment_grant(
        &self,
        principal: &BankAuthenticatedPrincipal,
        grant_identity: impl Into<String>,
        key: &u64,
        scope: &WorthQueryRequestScope,
    ) -> Result<
        WorthQueryApplicationMutationOutcome<
            ProgramDeploymentRevocationDenial,
            ProgramDeploymentRevocationChanged,
        >,
        WorthQueryApplicationRequestMutationDenial,
    > {
        self.request(principal, scope)
            .mutate(ProgramDeploymentRevocationIntent {
                input: ProgramDeploymentRevocationInput {
                    grant_identity: grant_identity.into(),
                },
            })
            .without_source()
            .idempotency(key)
            .execute_in_program(
                self.application_program(),
                ExecutionAllocationPolicy::SystemAllocation,
            )
    }
}
