//! Construction transfers original meaning into the native owning source lane.
use super::super::mutations;
use crate::{BankApplication, BankReadyMutation};
use bank_domain::schema::*;
use worth_query_host::facade::{
    application_entry::WorthQuerySourceOnlyOperation,
    declaration::application_program::ApplicationSourceCompletion,
};
/// The retired one-shot execution entrance is unavailable.
/// ```compile_fail,E0599
/// fn retired(runtime: &bank_server::BankIdentityRuntime, principal: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::CreatePersonalAccount, controls: bank_server::BankMutationControls) {
///     runtime.mutate(bank_server::mutations::create_personal_account(input)).as_principal(principal).controls(controls).execute();
/// }
/// ```
/// Owning construction returns no borrowed runtime or principal lifetime.
/// ```
/// fn construct(runtime: &bank_server::BankIdentityRuntime, principal: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::CreatePersonalAccount, controls: bank_server::BankMutationControls) {
///     let operation = runtime.mutate(bank_server::mutations::create_personal_account(input)).as_principal(principal).controls(controls).retain_operation();
///     let _ = operation;
/// }
/// ```
impl
    BankReadyMutation<
        '_,
        '_,
        mutations::CreatePersonalAccountMutation,
        CreatePersonalAccountOperation,
        Institution,
    >
{
    /// Owns the original input, conditions and key; every advance needs fresh request authority.
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            CreatePersonalAccount,
            BankApplication,
            crate::PersonalAccountCreationAction,
            ApplicationSourceCompletion,
        >,
        crate::BankOperationConstructionRefusal<CreatePersonalAccount>,
    > {
        let (scope, key) = self.controls.into_parts();
        self.runtime
            .request(self.principal, &scope)
            .mutate(self.mutation.input)
            .without_source()
            .preconditions(self.preconditions)
            .idempotency_owned(key)
            .map_err(|(builder, key, denial)| {
                crate::BankOperationConstructionRefusal::Capture(
                    builder.into_capture_refusal(key, denial),
                )
            })?
            .retain_operation::<BankApplication, crate::PersonalAccountCreationAction>(
                self.runtime.application_program(),
            )
            .map_err(
                |(description, denial)| crate::BankOperationConstructionRefusal::Declaration {
                    description,
                    denial,
                },
            )
    }
}
/// The retired one-shot execution entrance is unavailable.
/// ```compile_fail,E0599
/// fn retired(runtime: &bank_server::BankIdentityRuntime, principal: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::CreateBusinessAccount, controls: bank_server::BankMutationControls) {
///     runtime.mutate(bank_server::mutations::create_business_account(input)).as_principal(principal).controls(controls).execute();
/// }
/// ```
impl
    BankReadyMutation<
        '_,
        '_,
        mutations::CreateBusinessAccountMutation,
        CreateBusinessAccountOperation,
        Institution,
    >
{
    /// Owns the original input, conditions and key; every advance needs fresh request authority.
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            CreateBusinessAccount,
            BankApplication,
            crate::BusinessAccountCreationAction,
            ApplicationSourceCompletion,
        >,
        crate::BankOperationConstructionRefusal<CreateBusinessAccount>,
    > {
        let (scope, key) = self.controls.into_parts();
        self.runtime
            .request(self.principal, &scope)
            .mutate(self.mutation.input)
            .without_source()
            .preconditions(self.preconditions)
            .idempotency_owned(key)
            .map_err(|(builder, key, denial)| {
                crate::BankOperationConstructionRefusal::Capture(
                    builder.into_capture_refusal(key, denial),
                )
            })?
            .retain_operation::<BankApplication, crate::BusinessAccountCreationAction>(
                self.runtime.application_program(),
            )
            .map_err(
                |(description, denial)| crate::BankOperationConstructionRefusal::Declaration {
                    description,
                    denial,
                },
            )
    }
}
