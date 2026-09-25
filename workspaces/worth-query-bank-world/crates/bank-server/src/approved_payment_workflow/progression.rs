use bank_domain::{
    proposals::BankIdempotencyKey,
    schema::{ApprovePayment, ApprovedBusinessPaymentAdvanceIntent},
};
use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, WorkflowProgressOutcome, WorthQueryOrdinaryWorkflowRunProgress,
};

use super::{error::other_denial, BankApprovedPaymentWorkflow, BankApprovedPaymentWorkflowError};

impl BankApprovedPaymentWorkflow<'_, '_, '_> {
    pub fn advance(
        &self,
        instance: PublishedWorkflowInstanceRef,
        authority: ApprovePayment,
        command_key: &BankIdempotencyKey,
    ) -> Result<WorkflowProgressOutcome, BankApprovedPaymentWorkflowError> {
        self.runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_advance(self.runtime.approved_payment_workflow_runtime(), instance)
            .map(|request| request.execute())
            .map_err(other_denial)
    }

    pub fn run(
        &self,
        instance: PublishedWorkflowInstanceRef,
        authority: ApprovePayment,
        command_keys: &[BankIdempotencyKey],
    ) -> WorthQueryOrdinaryWorkflowRunProgress {
        self.runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .run_workflow(self.runtime.approved_payment_workflow_runtime(), instance)
            .idempotency_keys(command_keys)
            .execute()
    }
}
