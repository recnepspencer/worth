//! Explicit current native operation control, distinct from the ordinary owning entrance.
use super::*;
pub struct BankAdvancedApprovedPaymentWorkflow<'runtime, 'principal, 'scope> {
    pub(super) runtime: &'runtime BankIdentityRuntime,
    pub(super) principal: &'principal BankAuthenticatedPrincipal,
    pub(super) scope: &'scope WorthQueryRequestScope,
}
impl<'runtime, 'principal, 'scope>
    BankAdvancedApprovedPaymentWorkflow<'runtime, 'principal, 'scope>
{
    pub fn execute_operation(
        &self,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowOperation,
        operation: ApprovePayment,
        command_key: &BankIdempotencyKey,
    ) -> Result<BankApprovedPaymentApplyOutcome, BankApprovedPaymentWorkflowError> {
        let effect = self
            .runtime
            .request(self.principal, self.scope)
            .on_branch(instance.branch())
            .mutate(ApprovedBusinessPaymentApplyIntent {
                input: operation.clone(),
            })
            .idempotency(command_key)
            .for_workflow_operation(self.runtime.approved_payment_workflow_runtime(), required)
            .map_err(BankApprovedPaymentWorkflowError::OperationBinding)?
            .advanced()
            .execute_in_program(
                self.runtime
                    .approved_payment_workflow_runtime()
                    .program_runtime(),
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(BankApprovedPaymentWorkflowError::OperationMutation)?;
        Ok(match effect {
            WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => {
                BankApprovedPaymentApplyOutcome::Performed(BankApprovedPaymentPerformedOperation {
                    receipt,
                    newly_committed: true,
                })
            }
            WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt) => {
                BankApprovedPaymentApplyOutcome::Performed(BankApprovedPaymentPerformedOperation {
                    receipt,
                    newly_committed: false,
                })
            }
            WorthQueryApplicationMutationOutcome::PreviouslyCommitted(observation) => {
                BankApprovedPaymentApplyOutcome::PreviouslyCommitted(observation)
            }
            WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift => {
                BankApprovedPaymentApplyOutcome::IdempotencyIntentDrift
            }
            WorthQueryApplicationMutationOutcome::DomainDenied(denial) => {
                BankApprovedPaymentApplyOutcome::DomainDenied(denial)
            }
            WorthQueryApplicationMutationOutcome::Cancelled => {
                BankApprovedPaymentApplyOutcome::Cancelled
            }
            WorthQueryApplicationMutationOutcome::DeadlineExceeded => {
                BankApprovedPaymentApplyOutcome::DeadlineExceeded
            }
            WorthQueryApplicationMutationOutcome::Commit(outcome) => {
                BankApprovedPaymentApplyOutcome::Commit(outcome)
            }
        })
    }
}
