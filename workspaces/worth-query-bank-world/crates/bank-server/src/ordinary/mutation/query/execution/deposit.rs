//! Owning deposit construction transfers original meaning, never request authority.
use super::super::mutations;
use crate::{BankApplication, BankReadyMutation, DepositAction};
use bank_domain::schema::{BankSchema, Deposit, DepositOperation, Institution};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryOperationConstructionDenial, WorthQueryOperationDescription,
        WorthQuerySourceOnlyOperation,
    },
    declaration::application_program::ApplicationRetainedCommit,
};

impl<'runtime>
    BankReadyMutation<'runtime, '_, mutations::DepositMutation, DepositOperation, Institution>
{
    /// Retains the original deposit and key. Each advance needs a fresh request and history bound.
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            Deposit,
            BankApplication,
            DepositAction,
            ApplicationRetainedCommit,
        >,
        (
            WorthQueryOperationDescription<BankSchema, Deposit>,
            WorthQueryOperationConstructionDenial,
        ),
    > {
        let (scope, key) = self.controls.into_parts();
        self.runtime
            .request(self.principal, &scope)
            .mutate(self.mutation.input)
            .without_source()
            .preconditions(self.preconditions)
            .idempotency_owned(key)
            .retain_operation::<BankApplication, DepositAction>(self.runtime.application_program())
    }
}
