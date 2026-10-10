//! Bank's workflow operation advances through Query-owned performed custody.

use bank_domain::{
    proposals::BankIdempotencyKey,
    schema::{
        ApprovePayment, ApprovedBusinessPaymentAdvanceIntent, ApprovedBusinessPaymentApplyIntent,
    },
};
use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, RequiredWorkflowOperation, WorkflowProgressOutcome,
};
use worth_query_host::facade::primary_graph::WorthQueryRecoverySafeRetryAdmission;

use super::{
    BankApprovedPaymentPreparedRecovery, BankApprovedPaymentWorkflow,
    BankApprovedPaymentWorkflowError,
};

impl<'runtime, 'principal, 'scope> BankApprovedPaymentWorkflow<'runtime, 'principal, 'scope> {
    pub fn accept_applied(
        &self,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowOperation,
        authority: ApprovePayment,
        operation: ApprovePayment,
        perform_key: &BankIdempotencyKey,
        command_key: &BankIdempotencyKey,
    ) -> Result<WorkflowProgressOutcome, BankApprovedPaymentWorkflowError> {
        let operation = self
            .runtime
            .request(self.principal, self.scope)
            .on_branch(instance.branch())
            .mutate(ApprovedBusinessPaymentApplyIntent { input: operation })
            .idempotency(perform_key)
            .for_workflow_operation_recovery(
                self.runtime.approved_payment_workflow_runtime(),
                required,
            )
            .map_err(BankApprovedPaymentWorkflowError::OperationBinding)?;
        self.runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_advance(self.runtime.approved_payment_workflow_runtime(), instance)
            .map_err(BankApprovedPaymentWorkflowError::Advance)?
            .accept_operation_from_owner(required, operation)
            .map_err(BankApprovedPaymentWorkflowError::OperationOwnerAcceptance)
    }

    pub fn prepare_apply_recovery(
        &self,
        required: &RequiredWorkflowOperation,
        operation: ApprovePayment,
        perform_key: &BankIdempotencyKey,
    ) -> Result<BankApprovedPaymentPreparedRecovery<'runtime>, BankApprovedPaymentWorkflowError>
    {
        self.runtime
            .request(self.principal, self.scope)
            .on_branch(required.branch())
            .mutate(ApprovedBusinessPaymentApplyIntent { input: operation })
            .idempotency(perform_key)
            .for_workflow_operation_recovery(
                self.runtime.approved_payment_workflow_runtime(),
                required,
            )
            .map_err(BankApprovedPaymentWorkflowError::OperationBinding)?
            .prepare_workflow_operation_recovery_from_owner(required)
            .map_err(BankApprovedPaymentWorkflowError::OperationRecoveryPreparation)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn accept_recovered_applied(
        &self,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowOperation,
        authority: ApprovePayment,
        operation: ApprovePayment,
        perform_key: &BankIdempotencyKey,
        recovery: &WorthQueryRecoverySafeRetryAdmission,
        command_key: &BankIdempotencyKey,
    ) -> Result<WorkflowProgressOutcome, BankApprovedPaymentWorkflowError> {
        let operation = self
            .runtime
            .request(self.principal, self.scope)
            .on_branch(instance.branch())
            .mutate(ApprovedBusinessPaymentApplyIntent { input: operation })
            .idempotency(perform_key)
            .for_workflow_operation_recovery(
                self.runtime.approved_payment_workflow_runtime(),
                required,
            )
            .map_err(BankApprovedPaymentWorkflowError::OperationBinding)?;
        self.runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentAdvanceIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_advance(self.runtime.approved_payment_workflow_runtime(), instance)
            .map_err(BankApprovedPaymentWorkflowError::Advance)?
            .accept_recovered_operation_from_owner(required, operation, recovery)
            .map_err(BankApprovedPaymentWorkflowError::OperationOwnerAcceptance)
    }
}
