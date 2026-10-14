//! Pure owning construction; recorded issuer authority is admitted freshly on advance.
use crate::{
    BankApplication, BankAuthenticatedPrincipal, BankIdentityRuntime,
    ProgramDeploymentRevocationAction,
};
use bank_domain::schema::*;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::{
        WorthQueryOperationConstructionDenial, WorthQueryOperationDescription,
        WorthQuerySourceOnlyOperation,
    },
    declaration::{
        application_program::ApplicationSourceCompletion,
        application_schema::TypedMutationPreconditions,
    },
};
impl BankIdentityRuntime {
    /// Retains the original revocation and u64 key without retaining its requesting principal.
    /// The old one-shot revocation entrance is unavailable.
    /// ```compile_fail,E0599
    /// fn retired(runtime: &bank_server::BankIdentityRuntime) { let _ = runtime.revoke_program_deployment_grant; }
    /// ```
    /// Original typed conditions and key enter the same native owner.
    /// ```
    /// fn construct(runtime: &bank_server::BankIdentityRuntime, actor: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::ProgramDeploymentRevocationInput, key: u64, scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope) {
    ///     let _ = runtime.retain_program_deployment_revocation(actor, input, key, worth_query_host::facade::declaration::application_schema::TypedMutationPreconditions::default(), scope);
    /// }
    /// ```
    pub fn retain_program_deployment_revocation(
        &self,
        principal: &BankAuthenticatedPrincipal,
        input: ProgramDeploymentRevocationInput,
        key: u64,
        preconditions: TypedMutationPreconditions<
            BankSchema,
            ProgramDeploymentRevocationOperation,
            ProgramDeploymentGrant,
        >,
        scope: &WorthQueryRequestScope,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            ProgramDeploymentRevocationIntent,
            BankApplication,
            ProgramDeploymentRevocationAction,
            ApplicationSourceCompletion,
        >,
        (
            WorthQueryOperationDescription<BankSchema, ProgramDeploymentRevocationIntent>,
            WorthQueryOperationConstructionDenial,
        ),
    > {
        self.request(principal, scope)
            .mutate(ProgramDeploymentRevocationIntent { input })
            .without_source()
            .preconditions(preconditions)
            .idempotency_owned(key)
            .retain_operation::<BankApplication, ProgramDeploymentRevocationAction>(
                self.application_program(),
            )
    }
}
