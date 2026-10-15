//! Owning retained money construction preserves the existing installed delivery contract.
use super::super::mutations;
use crate::{BankApplication, BankReadyMutation};
use bank_domain::schema::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryOperationConstructionDenial, WorthQueryOperationDescription,
        WorthQuerySourceOnlyOperation,
    },
    declaration::application_program::ApplicationRetainedCommit,
};
impl
    BankReadyMutation<
        '_,
        '_,
        mutations::ReverseJournalMutation,
        ReverseJournalOperation,
        Institution,
    >
{
    /// Each advance requires fresh authority and an explicit native history bound.
    /// ```compile_fail,E0599
    /// fn retired(runtime: &bank_server::BankIdentityRuntime, actor: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::ReverseJournal, controls: bank_server::BankMutationControls) {
    ///     runtime.mutate(bank_server::mutations::reverse_journal(input)).as_principal(actor).controls(controls).execute();
    /// }
    /// ```
    /// ```
    /// fn construct(runtime: &bank_server::BankIdentityRuntime, actor: &bank_server::BankAuthenticatedPrincipal, input: bank_domain::schema::ReverseJournal, controls: bank_server::BankMutationControls) {
    ///     let _ = runtime.mutate(bank_server::mutations::reverse_journal(input)).as_principal(actor).controls(controls).retain_operation();
    /// }
    /// ```
    pub fn retain_operation(
        self,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            BankSchema,
            ReverseJournal,
            BankApplication,
            crate::ReverseJournalAction,
            ApplicationRetainedCommit,
        >,
        (
            WorthQueryOperationDescription<BankSchema, ReverseJournal>,
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
            .retain_operation::<BankApplication, crate::ReverseJournalAction>(
                self.runtime.application_program(),
            )
    }
}
