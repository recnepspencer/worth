use bank_domain::{
    proposals::BankIdempotencyKey,
    schema::{
        ApprovePayment, ApprovedBusinessPaymentAdvanceIntent,
        ApprovedBusinessPaymentInstanceStartIntent,
    },
};
use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef,
    WorkflowInstanceCancellationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorthQueryOrdinaryWorkflowRunProgress,
};

use super::{BankApprovedPaymentWorkflow, BankApprovedPaymentWorkflowError};

impl<'runtime> BankApprovedPaymentWorkflow<'runtime, '_, '_> {
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
            .map_err(BankApprovedPaymentWorkflowError::Advance)
    }

    /// Prepares ending a live payment workflow where it stands. A payment the
    /// rail owner still holds is refused until it is accepted; the outcome
    /// then names it. The cancellation commits against the basis it read, so
    /// a payment the rail takes in between leaves it stale.
    pub fn prepare_cancellation(
        &self,
        instance: PublishedWorkflowInstanceRef,
        authority: ApprovePayment,
        command_key: &BankIdempotencyKey,
    ) -> Result<BankApprovedPaymentCancellation<'runtime>, BankApprovedPaymentWorkflowError> {
        let request = self
            .runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentInstanceStartIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_instance_cancellation(
                self.runtime.approved_payment_workflow_runtime(),
                instance,
            )
            .map_err(BankApprovedPaymentWorkflowError::InstanceCancellation)?;
        Ok(BankApprovedPaymentCancellation {
            execute: Box::new(move || request.execute()),
        })
    }

    /// Ends a live payment workflow and continues it on `target`, a later
    /// definition of the same workflow, from the node `resume_at` names. A
    /// payment the rail owner still holds is refused until it is accepted.
    pub fn migrate(
        &self,
        instance: PublishedWorkflowInstanceRef,
        target: PublishedWorkflowDefinitionRef,
        resume_at: &str,
        authority: ApprovePayment,
        command_key: &BankIdempotencyKey,
    ) -> Result<WorkflowInstanceStartOutcome, BankApprovedPaymentWorkflowError> {
        self.runtime
            .request(self.principal, self.scope)
            .mutate(ApprovedBusinessPaymentInstanceStartIntent { input: authority })
            .without_source()
            .idempotency(command_key)
            .prepare_workflow_instance_migration(
                self.runtime.approved_payment_workflow_runtime(),
                instance,
                target,
                resume_at,
            )
            .map(|request| request.execute())
            .map_err(BankApprovedPaymentWorkflowError::InstanceMigration)
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

/// One prepared payment workflow cancellation, not yet committed.
pub struct BankApprovedPaymentCancellation<'runtime> {
    execute: Box<dyn FnOnce() -> WorkflowInstanceCancellationOutcome + 'runtime>,
}

impl BankApprovedPaymentCancellation<'_> {
    pub fn execute(self) -> WorkflowInstanceCancellationOutcome {
        (self.execute)()
    }
}
