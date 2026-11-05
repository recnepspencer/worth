//! The actual Relational validation, under the admitted request control.
use super::*;
use crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl;

impl WorthQueryPrimaryGraphProvider {
    pub(super) fn validate_relational_candidate(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        batch: worth_relational::facade::transactions::WorkerIntentBatch,
        content: &worth_relational::facade::content::RelationalContentAttachments,
        branch: &worth_relational::facade::history::BranchId,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        application_touches: &worth_query_installation::facade::WorthQueryOperationTouchContract,
        aftermath_causality: Option<
            &crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
        >,
        restoration: Option<worth_relational::facade::mvcc::RequiredOutputPublication>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        (worth_relational::facade::mvcc::ValidatedRelationalProposal, Option<crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationOwner>),
        WorthQueryInvariantExecutionFailure,
    >{
        #[cfg(not(test))]
        let _ = application_touches;
        let batch = if self.take_relational_invariant_violation() {
            batch.push(invariant_violation_probe())
        } else {
            batch
        };
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
        // The caller owns this before Native can snapshot any predicate posting.
        // Native only borrows the factory and retains bundled weak receipts.
        let factory = if self.graph.managed_derived_views().has_predicate_routing() {
            let fence = self.live_delivery.begin_predicate_preparation(execution, product.observation())
                .map_err(|denial| match denial {
                    crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial::Execution(cause) => context_execution_failure(cause),
                    crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial::Unavailable(detail) => {
                        WorthQueryInvariantExecutionFailure::new(WorthQueryInvariantExecutionDenialKind::ManagedPredicatePreparationUnavailable, detail)
                    }
                })?;
            Some(crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationFactory::new(
                execution, product.observation(), fence,
            ).map_err(context_execution_failure)?)
        } else {
            None
        };
        let candidate = self.graph.with_runtime_mut(execution, |runtime| {
            if let Some(pending) = aftermath_causality {
                let observed_parent = basis
                    .observation()
                    .commit_receipt()
                    .cloned()
                    .ok_or_else(aftermath_failure)?;
                if pending.parent() != &observed_parent || pending.target_branch() != branch {
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
                .attach_content(execution, content)
                .map_err(|cause| {
                    WorthQueryInvariantExecutionFailure::content(
                        cause,
                        "Native candidate content attachment refused",
                    )
                })?;
            transaction
                .push_batch(batch, allocation_control.policy())
                .map_err(map_transaction_staging_failure)?;
            self.fault_port.candidate_staged_for_validation();
            Ok::<_, WorthQueryInvariantExecutionFailure>(match &factory {
                Some(factory) => runtime.validate_branch_transaction_with_source_context(
                    execution,
                    transaction,
                    factory,
                ),
                None => transaction.validate(execution, runtime),
            })
        });
        let candidate = candidate?.map_err(|failure| map_failed_candidate(failure, &control))?;
        let candidate = match restoration {
            Some(publication) => candidate
                .bind_required_output_publication(
                    &self
                        .graph
                        .application_publication
                        .as_ref()
                        .ok_or_else(owner_failure)?
                        .native,
                    publication,
                )
                .map_err(map_validation_failure)?,
            None => candidate,
        };
        validate_owner_evidence(candidate.invariant_evidence(), branch)?;
        let predicate = factory.map(|factory| {
            factory.promote(execution, candidate.managed_source_preparation_metadata().ok_or_else(owner_failure)?)
                .map_err(|cause| match cause {
                    worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial::Source(
                        worth_relational::facade::runtime::DerivedRelationReadDenial::Request(cause),
                    ) => context_execution_failure(cause),
                    cause => WorthQueryInvariantExecutionFailure::managed_source(cause, "managed predicate preparation did not promote its accepted Native round"),
                })
        }).transpose()?;
        Ok((candidate, predicate))
    }
}
fn context_execution_failure(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> WorthQueryInvariantExecutionFailure {
    super::super::invariant_execution_failure::map_execution_failure(
        worth_relational::facade::transactions::CommitExecutionDenial {
            kind: worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(cause),
            partition_identity: None,
        },
    )
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
    ) && let Some(interruption) = control.relational().interruption()
    {
        return super::super::invariant_execution_failure::request_interruption(interruption);
    }
    map_validation_failure(failure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_caller_keeps_validation_work_stop_and_stage() {
        use worth_relational::facade::transactions::{
            CommitExecutionDenial, CommitExecutionDenialKind, RelationalExecutionDenialCause,
        };
        use {
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial,
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialStage,
            crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial as Resource,
        };
        use {
            crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind as ExecutionKind,
            crate::domain_computation::WorthQueryInvariantExecutionDenialKind as Kind,
        };
        let source = worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new();
        let request =
            worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(
                Some(std::time::Instant::now() + std::time::Duration::from_secs(60)),
                source.token(), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
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
