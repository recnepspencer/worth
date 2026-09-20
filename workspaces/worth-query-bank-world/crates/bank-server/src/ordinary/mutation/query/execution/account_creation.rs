use bank_domain::schema::{
    BankSchema, CreateBusinessAccountOperation, CreatePersonalAccountOperation, Institution,
};
use worth_query_host::facade::declaration::application_schema::TypedMutationPreconditions;

use super::super::mutations;
use crate::ordinary::mutation::{
    BankBusinessAccountCreationExecution, BankPersonalAccountCreationExecution,
};
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime, BankReadyMutation};

impl
    BankReadyMutation<
        '_,
        '_,
        mutations::CreatePersonalAccountMutation,
        CreatePersonalAccountOperation,
        Institution,
    >
{
    pub fn execute(self) -> BankPersonalAccountCreationExecution {
        execute_personal(
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
        mutations::CreateBusinessAccountMutation,
        CreateBusinessAccountOperation,
        Institution,
    >
{
    pub fn execute(self) -> BankBusinessAccountCreationExecution {
        execute_business(
            self.runtime,
            self.principal,
            self.preconditions,
            &self.controls,
            self.mutation.input,
        )
    }
}

fn execute_personal(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    preconditions: TypedMutationPreconditions<
        BankSchema,
        CreatePersonalAccountOperation,
        Institution,
    >,
    controls: &crate::BankMutationControls,
    input: bank_domain::schema::CreatePersonalAccount,
) -> BankPersonalAccountCreationExecution {
    runtime
        .request(principal, controls.request())
        .mutate(input)
        .preconditions(preconditions)
        .idempotency(controls.idempotency_key())
        .execute_in_selected_program(runtime.application_program())
}

fn execute_business(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    preconditions: TypedMutationPreconditions<
        BankSchema,
        CreateBusinessAccountOperation,
        Institution,
    >,
    controls: &crate::BankMutationControls,
    input: bank_domain::schema::CreateBusinessAccount,
) -> BankBusinessAccountCreationExecution {
    runtime
        .request(principal, controls.request())
        .mutate(input)
        .preconditions(preconditions)
        .idempotency(controls.idempotency_key())
        .execute_in_selected_program(runtime.application_program())
}
