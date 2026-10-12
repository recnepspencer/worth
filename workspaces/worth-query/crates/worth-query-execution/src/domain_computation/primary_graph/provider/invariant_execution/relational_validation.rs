//! The actual Relational validation, under the admitted request control.
use super::*;
use crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl;

impl WorthQueryPrimaryGraphProvider {
    pub(super) fn validate_relational_candidate(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        batch: worth_relational::facade::transactions::WorkerIntentBatch,
        branch: &worth_relational::facade::history::BranchId,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        application_touches: &worth_query_installation::facade::WorthQueryOperationTouchContract,
        aftermath_causality: Option<
            &crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        worth_relational::facade::mvcc::ValidatedRelationalProposal,
        WorthQueryInvariantExecutionFailure,
    > {
        #[cfg(not(test))]
        let _ = application_touches;
        let batch = if self.take_relational_invariant_violation() {
            batch.push(invariant_violation_probe())
        } else {
            batch
        };
        #[cfg(test)]
        let batch = if self.take_undeclared_application_touch() {
            batch.push(undeclared_application_touch_probe(
                &self.graph.layout,
                application_touches,
            )?)
        } else {
            batch
        };
        let basis = product.observation().basis().relational_basis();
        if basis.identity().branch_id() != branch {
            return Err(owner_failure());
        }
        let control = super::validation_control::ValidationRequestControl::new(request);
        let allocation_control = RequestAllocationControl::new(request, allocation_policy);
        let candidate = self.graph.with_runtime_mut(execution, |runtime| {
            if let Some(pending) = aftermath_causality {
                let observed_parent = basis
                    .observation()
                    .commit_receipt()
                    .cloned()
                    .ok_or_else(aftermath_failure)?;
                if pending.parent() != &observed_parent {
                    return Err(aftermath_failure());
                }
            }
            let mut transaction = runtime
                .begin_branch_transaction_with_control(
                    basis,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                    control.relational().clone(),
                )
                .map_err(map_transaction_admission_failure)?;
            transaction
                .push_batch(batch, allocation_control.policy())
                .map_err(map_transaction_staging_failure)?;
            #[cfg(test)]
            self.fault_port.candidate_staged_for_validation();
            Ok::<_, WorthQueryInvariantExecutionFailure>(transaction.validate(execution, runtime))
        });
        let candidate = candidate?.map_err(|failure| map_failed_candidate(failure, &control))?;
        validate_owner_evidence(candidate.invariant_evidence(), branch)?;
        Ok(candidate)
    }
}

fn map_failed_candidate(
    failure: worth_relational::facade::mvcc::TransactionCommitError,
    control: &super::validation_control::ValidationRequestControl,
) -> WorthQueryInvariantExecutionFailure {
    // The stage consult can notice the same caller interruption before
    // native validation reaches its operation-control safe point.
    use worth_relational::facade::transactions::{
        CommitExecutionDenial, CommitExecutionDenialKind, RelationalExecutionDenialCause as Cause,
    };
    if matches!(
        failure,
        worth_relational::facade::mvcc::TransactionCommitError::Execution {
            denial: CommitExecutionDenial {
                kind: CommitExecutionDenialKind::Cause(Cause::Cancelled | Cause::DeadlineElapsed),
                ..
            },
            ..
        }
    ) {
        if let Some(interruption) = control.relational().interruption() {
            return super::super::invariant_execution_failure::request_interruption(interruption);
        }
    }
    map_validation_failure(failure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_caller_keeps_validation_work_stop_and_stage() {
        use crate::domain_computation::primary_graph::{
            WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialStage,
            WorthQueryManagedComputationResourceDenial as Resource,
        };
        use crate::domain_computation::{
            WorthQueryInvariantExecutionDenialKind as Kind,
            WorthQueryProviderSessionDenialKind as ExecutionKind,
        };
        use worth_relational::facade::transactions::{
            CommitExecutionDenial, CommitExecutionDenialKind, RelationalExecutionDenialCause,
        };
        let source = worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new();
        let request =
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(
                Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
                source.token(),
            );
        let control = super::super::validation_control::ValidationRequestControl::new(&request);
        source.cancel();
        assert_eq!(
            control.relational().interruption(),
            Some(worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled)
        );
        let failure = worth_relational::facade::mvcc::TransactionCommitError::Execution {
            denial: CommitExecutionDenial {
                kind: CommitExecutionDenialKind::Cause(
                    RelationalExecutionDenialCause::WorkExhausted,
                ),
                partition_identity: Some(23),
            },
            context: worth_relational::facade::errors::ErrorContext::new(
                worth_relational::facade::errors::RelationalSubsystem::Transaction,
                worth_relational::facade::errors::ErrorOperation::Validate,
            ),
            commit_log: worth_relational::facade::transactions::CommitLog::new(),
        };
        let denial = WorthQueryApplicationCommitDenial::invariant_execution_denied(
            WorthQueryApplicationCommitDenialStage::InvariantExecution,
            map_failed_candidate(failure, &control),
        );
        assert_eq!(
            denial.stage(),
            WorthQueryApplicationCommitDenialStage::InvariantExecution
        );
        assert_eq!(
            denial.invariant_execution_failure().unwrap().kind(),
            Kind::ExecutionDenied(ExecutionKind::ExecutionResource {
                denial: Resource::WorkExhausted,
                partition_identity: Some(23),
                policy_ancestor: None
            })
        );
    }
}
