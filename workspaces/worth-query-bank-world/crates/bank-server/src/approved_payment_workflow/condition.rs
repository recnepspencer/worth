use bank_domain::{
    proposals::BankIdempotencyKey,
    queries::{payment_amount, PaymentAmountQueryBinding},
    schema::{ApprovePayment, ApprovedBusinessPaymentAdvanceIntent},
};
use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, RequiredWorkflowCondition, WorkflowProgressOutcome,
};

use super::{BankApprovedPaymentWorkflow, BankApprovedPaymentWorkflowError};

impl BankApprovedPaymentWorkflow<'_, '_, '_> {
    /// Settles the approval limit `required` names: reads the payment's
    /// current amount and accepts it as the condition's only operand. A
    /// payment within the limit continues to approval; one over it is
    /// rejected.
    pub fn accept_approval_limit(
        &self,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowCondition,
        authority: ApprovePayment,
        command_key: &BankIdempotencyKey,
    ) -> Result<WorkflowProgressOutcome, BankApprovedPaymentWorkflowError> {
        let amount = self
            .runtime
            .request(self.principal, self.scope)
            .query(payment_amount(authority.payment))
            .execute()
            .map_err(BankApprovedPaymentWorkflowError::ConditionRead)?;
        self.runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_advance(self.runtime.approved_payment_workflow_runtime(), instance)
            .map_err(BankApprovedPaymentWorkflowError::Advance)?
            .condition(required)
            .operand::<PaymentAmountQueryBinding, _>(
                crate::application_definition::APPROVAL_LIMIT_OPERAND,
                amount,
            )
            .accept()
            .map_err(BankApprovedPaymentWorkflowError::ConditionAcceptance)
    }
}
