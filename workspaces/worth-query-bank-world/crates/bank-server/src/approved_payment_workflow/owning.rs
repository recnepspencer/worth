//! Ordinary workflow acceptance preserves the completed effect and native cleanup on refusal.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryCompletedWorkflowOperation, WorthQueryWorkflowAcceptance,
};
/// Ordinary execution cannot skip the owning workflow lifecycle.
/// ```compile_fail,E0599
/// fn cannot_execute(owner: worth_query_host::facade::application_entry::WorthQueryWorkflowOperation<bank_domain::schema::BankSchema,bank_domain::schema::ApprovedBusinessPaymentApplyIntent,bank_server::BankApplication,bank_server::ApprovedPaymentEffectAction>) {
///     owner.execute_in_program();
/// }
/// ```
/// The actual concrete owner admits a fresh advancement.
/// ```no_run
/// fn advance(owner: worth_query_host::facade::application_entry::WorthQueryWorkflowOperation<bank_domain::schema::BankSchema,bank_domain::schema::ApprovedBusinessPaymentApplyIntent,bank_server::BankApplication,bank_server::ApprovedPaymentEffectAction>,request:&worth_query_host::facade::application_entry::WorthQueryApplicationRequest<'_,'_,'_,bank_domain::schema::BankSchema>) { let _=owner.advance(request); }
/// ```
/// Observation is not original operation authority.
/// ```compile_fail,E0308
/// fn cannot_approve(workflow:&bank_server::BankApprovedPaymentWorkflow<'_,'_,'_>,observation:&worth_query_host::facade::application_entry::RequiredWorkflowAssessment,input:bank_domain::schema::ApprovePayment,key:bank_domain::proposals::BankIdempotencyKey) {
///     workflow.retain_apply(observation,input,key);
/// }
/// ```
/// The real native operation requirement is the lawful counterpart.
/// ```no_run
/// fn bind(workflow:&bank_server::BankApprovedPaymentWorkflow<'_,'_,'_>,required:&worth_query_host::facade::application_entry::RequiredWorkflowOperation,input:bank_domain::schema::ApprovePayment,key:bank_domain::proposals::BankIdempotencyKey) { let _=workflow.retain_apply(required,input,key); }
/// ```
/// Terminal custody cannot return to effectful advancement.
/// ```compile_fail,E0599
/// fn cannot_advance(terminal:worth_query_host::facade::application_entry::WorthQueryTerminalWorkflowOperation<bank_domain::schema::BankSchema,bank_domain::schema::ApprovedBusinessPaymentApplyIntent,bank_server::BankApplication,bank_server::ApprovedPaymentEffectAction>) { terminal.advance(); }
/// ```
/// Its actual lawful entrance is fresh close, retaining itself on refusal.
/// ```no_run
/// fn close(terminal:worth_query_host::facade::application_entry::WorthQueryTerminalWorkflowOperation<bank_domain::schema::BankSchema,bank_domain::schema::ApprovedBusinessPaymentApplyIntent,bank_server::BankApplication,bank_server::ApprovedPaymentEffectAction>,request:&worth_query_host::facade::application_entry::WorthQueryApplicationRequest<'_,'_,'_,bank_domain::schema::BankSchema>) { let _=terminal.close(request); }
/// ```
pub type BankCompletedApprovedPayment = WorthQueryCompletedWorkflowOperation<
    BankSchema,
    ApprovedBusinessPaymentApplyIntent,
    crate::BankApplication,
    crate::ApprovedPaymentEffectAction,
>;
impl BankApprovedPaymentWorkflow<'_, '_, '_> {
    pub fn accept_owned_applied(
        &self,
        instance: PublishedWorkflowInstanceRef,
        authority: ApprovePayment,
        operation: BankCompletedApprovedPayment,
        command_key: &BankIdempotencyKey,
    ) -> Result<
        WorthQueryWorkflowAcceptance<BankCompletedApprovedPayment>,
        (
            BankCompletedApprovedPayment,
            BankApprovedPaymentWorkflowError,
        ),
    > {
        let prepared = self
            .runtime
            .request(self.principal, self.scope)
            .on_branch(instance.branch())
            .mutate(bank_domain::schema::ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_advance(self.runtime.approved_payment_workflow_runtime(), instance);
        let prepared = match prepared {
            Ok(prepared) => prepared,
            Err(cause) => {
                return Err((operation, BankApprovedPaymentWorkflowError::Advance(cause)))
            }
        };
        prepared
            .accept_owned_operation(operation)
            .map_err(|(operation, cause)| {
                (
                    operation,
                    BankApprovedPaymentWorkflowError::OperationOwnerAcceptance(Box::new(cause)),
                )
            })
    }
}
