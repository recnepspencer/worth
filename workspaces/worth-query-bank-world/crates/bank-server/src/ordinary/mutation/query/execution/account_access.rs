use bank_domain::schema::{
    Account, BankSchema, GrantAccountAuthorizationOperation, RevokeAccountAuthorizationOperation,
};
use worth_query_host::facade::declaration::application_schema::TypedMutationPreconditions;

use super::super::mutations;
use crate::ordinary::mutation::BankAccountAccessExecution;
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankReadyMutation};

impl
    BankReadyMutation<
        '_,
        '_,
        mutations::GrantAccountAccessMutation,
        GrantAccountAuthorizationOperation,
        Account,
    >
{
    pub fn execute(self) -> BankAccountAccessExecution {
        execute_grant(
            self.runtime,
            self.principal,
            self.preconditions,
            &self.controls,
            self.mutation.input,
        )
    }
}

impl
    BankReadyMutation<
        '_,
        '_,
        mutations::RevokeAccountAccessMutation,
        RevokeAccountAuthorizationOperation,
        Account,
    >
{
    pub fn execute(self) -> BankAccountAccessExecution {
        execute_revoke(
            self.runtime,
            self.principal,
            self.preconditions,
            &self.controls,
            self.mutation.input,
        )
    }
}

fn execute_grant(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    preconditions: TypedMutationPreconditions<
        BankSchema,
        GrantAccountAuthorizationOperation,
        Account,
    >,
    controls: &crate::BankMutationControls,
    input: bank_domain::schema::GrantAccountAuthorization,
) -> BankAccountAccessExecution {
    runtime
        .request(principal, controls.request())
        .mutate(input)
        .preconditions(preconditions)
        .idempotency(controls.idempotency_key())
        .execute_in_selected_program(runtime.application_program())
}

fn execute_revoke(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    preconditions: TypedMutationPreconditions<
        BankSchema,
        RevokeAccountAuthorizationOperation,
        Account,
    >,
    controls: &crate::BankMutationControls,
    input: bank_domain::schema::RevokeAccountAuthorization,
) -> BankAccountAccessExecution {
    runtime
        .request(principal, controls.request())
        .mutate(input)
        .preconditions(preconditions)
        .idempotency(controls.idempotency_key())
        .execute_in_selected_program(runtime.application_program())
}
