//! Owning send_money construction transfers original meaning, never request authority.
use super::super::mutations;
use crate::{BankApplication, BankReadyMutation, SendMoneyAction};
use bank_domain::schema::{Account, BankSchema, SendMoney, SendMoneyOperation};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryOperationConstructionDenial, WorthQueryOperationDescription,
        WorthQuerySourceOnlyOperation,
    },
    declaration::application_program::ApplicationRetainedCommit,
};

/// One-shot money execution is retired.
/// ```compile_fail,E0599
/// fn retired(runtime: &bank_server::BankIdentityRuntime, principal: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::SendMoney, controls: bank_server::BankMutationControls) {
///     runtime.mutate(bank_server::mutations::send_money(input)).as_principal(principal).controls(controls).execute();
/// }
/// ```
/// Owning construction keeps original input, key and typed conditions.
/// ```
/// fn construct(runtime: &bank_server::BankIdentityRuntime, principal: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::SendMoney, controls: bank_server::BankMutationControls) {
///     let _operation = runtime.mutate(bank_server::mutations::send_money(input)).as_principal(principal).controls(controls).retain_operation();
/// }
/// ```
impl<'runtime>
    BankReadyMutation<'runtime, '_, mutations::SendMoneyMutation, SendMoneyOperation, Account>
{
    /// Retains the original send_money and key. Each advance needs a fresh request and history bound.
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            SendMoney,
            BankApplication,
            SendMoneyAction,
            ApplicationRetainedCommit,
        >,
        (
            WorthQueryOperationDescription<BankSchema, SendMoney>,
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
            .retain_operation::<BankApplication, SendMoneyAction>(
                self.runtime.application_program(),
            )
    }
}
