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
/// fn retired(runtime: &bank_server::BankIdentityRuntime, principal: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::GrantAccountAuthorization, controls: bank_server::BankMutationControls) {
///     runtime.mutate(bank_server::mutations::grant_account_access(input)).as_principal(principal).controls(controls).execute();
/// }
/// ```
impl
    BankReadyMutation<
        '_,
        '_,
        mutations::GrantAccountAccessMutation,
        GrantAccountAuthorizationOperation,
        Account,
    >
{
    /// Owns the original input, conditions and key; every advance needs fresh request authority.
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            GrantAccountAuthorization,
            BankApplication,
            crate::AccountAccessGrantAction,
            ApplicationSourceCompletion,
        >,
        crate::BankOperationConstructionRefusal<GrantAccountAuthorization>,
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
            .retain_operation::<BankApplication, crate::AccountAccessGrantAction>(
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
/// fn retired(runtime: &bank_server::BankIdentityRuntime, principal: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::RevokeAccountAuthorization, controls: bank_server::BankMutationControls) {
///     runtime.mutate(bank_server::mutations::revoke_account_access(input)).as_principal(principal).controls(controls).execute();
/// }
/// ```
impl
    BankReadyMutation<
        '_,
        '_,
        mutations::RevokeAccountAccessMutation,
        RevokeAccountAuthorizationOperation,
        Account,
    >
{
    /// Owns the original input, conditions and key; every advance needs fresh request authority.
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            RevokeAccountAuthorization,
            BankApplication,
            crate::AccountAccessRevocationAction,
            ApplicationSourceCompletion,
        >,
        crate::BankOperationConstructionRefusal<RevokeAccountAuthorization>,
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
            .retain_operation::<BankApplication, crate::AccountAccessRevocationAction>(
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
