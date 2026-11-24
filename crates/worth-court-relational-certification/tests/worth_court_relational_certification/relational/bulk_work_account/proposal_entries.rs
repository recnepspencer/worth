//! Account for validation and publication through the caller's commit entries.
use super::*;
use worth_execution::CancellationSource;
use worth_relational::facade::{
    commit_strategies::{
        CanonicalStrategyCommitRequest, CanonicalStrategyOutputArtifact,
        CommitStrategyExecutionRegistration, CommitStrategyExecutor, CommitStrategyId,
        CommitStrategyRegistration, IntentReconciliationStrategy, NativeStrategyCommitRequest,
        PersistentArtifactName, StrategyCallerProvenance, StrategyExecutionResult,
        StrategyExecutorFailure, StrategyMutationProgram, StrategyObservationContext,
        StrategyOutputSchemaName, StrategyRequestOrigin,
    },
    mvcc::{RelationalPublicationOutcome, RelationalTransactionIntent},
    runtime::RelationalRuntimeApi,
    transactions::RelationalExecutionDenialCause,
};
use worth_testkit_relational::durable_publication::{batch_create, schema};

#[derive(Clone, Copy, Debug)]
enum Entry {
    Ordinary,
    Batch,
    Split,
    Strategy,
    Prepared,
    CancelledAfterValidation,
}
const ENTRIES: [Entry; 5] = [
    Entry::Ordinary,
    Entry::Batch,
    Entry::Split,
    Entry::Strategy,
    Entry::Prepared,
];

// Fixture funding, not a price for an implementation's incidental dispatches.
const FUNDED_WORK_CEILING: u64 = 1_000_000;

struct CreateDeployment;

impl CommitStrategyExecutor for CreateDeployment {
    fn execute(
        &self,
        _execution: &ExecutionRequest<'_, '_>,
        _request: &CanonicalStrategyCommitRequest,
        _observation: &StrategyObservationContext<'_>,
    ) -> Result<StrategyExecutionResult, StrategyExecutorFailure> {
        Ok(StrategyExecutionResult::new(
            CanonicalStrategyOutputArtifact::new(
                StrategyOutputSchemaName::new(
                    IntentReconciliationStrategy::DEFAULT_OUTPUT_SCHEMA_NAME,
                ),
                b"status=planned".to_vec(),
                PersistentArtifactName::new("strategy.intent.reconcile.output"),
            ),
            StrategyMutationProgram::new(vec![batch_create("deployment-a")]),
        ))
    }
}

fn commit(
    entry: Entry,
    ceiling: u64,
    observe_whole_request: bool,
    cancel_before_entry: bool,
) -> (Result<CommitResult, TransactionCommitError>, u64) {
    let setup = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let setup_request = ExecutionRequest::serial(&setup);
    let descriptor = IntentReconciliationStrategy::descriptor(CommitStrategyId(501));
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema())
        .commit_strategy(CommitStrategyRegistration::new(descriptor.clone()).unwrap())
        .commit_strategy_executor(CommitStrategyExecutionRegistration::new(
            &descriptor,
            CreateDeployment,
        ))
        .build()
        .unwrap();
    let identity = runtime.main_branch_identity();
    let (before, basis) = runtime.observe_branch(&identity).unwrap();
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let facade = runtime.commit_strategies();
    let strategy = facade
        .canonicalize_request(&NativeStrategyCommitRequest::from_native_canonical_bytes(
            descriptor.semantic_name().clone(),
            b"create-deployment".to_vec(),
            StrategyCallerProvenance {
                request_origin: StrategyRequestOrigin::Test,
                actor_identity: None,
                correlation_id: None,
            },
        ))
        .unwrap();
    let draft = facade
        .execute(&strategy, &snapshot, &setup_request)
        .unwrap();
    runtime.snapshots().release_snapshot(&snapshot).unwrap();

    let mut authority = runtime.commit_strategies_authority();
    let mut lowered = None;
    let mut staged = None;
    let intent = RelationalTransactionIntent::ordinary();
    if matches!(entry, Entry::Strategy) {
        lowered = Some(
            authority
                .lower_execution(&setup_request, &runtime, &strategy, &draft, &basis, intent)
                .unwrap(),
        );
    } else {
        let mut transaction = runtime.begin_branch_transaction(&basis, intent).unwrap();
        for batch in draft.mutation_program().worker_batches() {
            transaction
                .push_batch(batch.clone(), setup_request.allocation_policy())
                .unwrap();
        }
        transaction
            .prepare_merged_plan(&setup_request, &runtime)
            .unwrap();
        if matches!(entry, Entry::Batch) {
            for label in ["empty-left", "empty-right"] {
                transaction
                    .push_batch(
                        WorkerIntentBatch::new(label),
                        setup_request.allocation_policy(),
                    )
                    .unwrap();
            }
        }
        staged = Some(transaction);
    }

    let cancellation = CancellationSource::new();
    let lease = super::execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(ceiling)),
            ),
            deadline: None,
            cancellation: cancellation.token(),
        })
        .unwrap();
    let execution = ExecutionRequest::leased(&lease);
    let run = || {
        super::record_work(|| {
            if cancel_before_entry {
                cancellation.cancel();
            }
            let candidate = match entry {
                Entry::Strategy => {
                    let proposal =
                        authority.validate_lowered_plan(&execution, &runtime, lowered.unwrap())?;
                    authority.prepare_validated_commit(&execution, &runtime, proposal)?
                }
                Entry::Ordinary | Entry::Batch => {
                    return runtime.commit_branch_transaction(&execution, staged.unwrap());
                }
                Entry::Prepared => {
                    runtime.prepare_branch_transaction(&execution, staged.unwrap())?
                }
                Entry::Split | Entry::CancelledAfterValidation => {
                    let proposal =
                        runtime.validate_branch_transaction(&execution, staged.unwrap())?;
                    if matches!(entry, Entry::CancelledAfterValidation) {
                        cancellation.cancel();
                    }
                    runtime.prepare_validated_proposal(&execution, proposal)?
                }
            };
            match runtime
                .publication_port()
                .compare_and_publish(&execution, candidate)
            {
                RelationalPublicationOutcome::Performed(performed) => {
                    runtime.settle_performed_publication(performed)
                }
                RelationalPublicationOutcome::RequestStopped(stop) => Err(stop.into()),
                other => panic!("{entry:?}: unexpected publication outcome: {other:?}"),
            }
        })
    };
    let ((outcome, charges), whole_work) = if observe_whole_request {
        let (result, report) = execution
            .run(ExecutionWorkCeiling::new(None), |_| run())
            .unwrap();
        (result, Some(report.charged_work()))
    } else {
        (run(), None)
    };
    if let Ok(ref result) = outcome {
        assert_eq!(
            result.changed_records.len(),
            1,
            "{entry:?}: fixed proposal completes"
        );
    } else {
        assert_eq!(
            runtime.observe_branch(&identity).unwrap().0,
            before,
            "{entry:?}: refusal moves no head"
        );
    }
    (
        outcome,
        whole_work.unwrap_or_else(|| charges.into_iter().sum()),
    )
}

#[test]
fn one_work_account_bounds_each_entry_with_its_original_canonical_evidence() {
    for entry in ENTRIES {
        // Strategy retains its actual StrategyCommitArtifactBundle; the other
        // entries discard that decoration. Hashing those distinct original
        // canonical subjects may have different byte costs.
        let (reference, required) = commit(entry, FUNDED_WORK_CEILING, true, false);
        reference.unwrap();
        assert!(required > 0 && required < FUNDED_WORK_CEILING);
        let (unwrapped, recorded) = commit(entry, required, false, false);
        unwrapped.unwrap_or_else(|error| {
            panic!("{entry:?}: exact invoking account {required}: {error:?}")
        });
        assert_eq!(
            recorded, required,
            "{entry:?}: every original SDK charge settles into the same account"
        );
        let (wrapped, charged) = commit(entry, required, true, false);
        wrapped.unwrap_or_else(|error| {
            panic!("{entry:?}: exact ancestor and invoking accounts {required}: {error:?}")
        });
        assert_eq!(charged, required);
        let refused = commit(entry, required - 1, false, false).0;
        assert!(
            matches!(refused, Err(TransactionCommitError::Execution { denial, .. })
            if denial.kind == CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::WorkExhausted)),
            "{entry:?}: one shared ceiling below {required} must refuse"
        );
    }
}

#[test]
fn equivalent_proposal_entries_preserve_mutation_results() {
    let (ordinary, _) = commit(Entry::Ordinary, FUNDED_WORK_CEILING, true, false);
    let ordinary = ordinary.unwrap();
    for entry in ENTRIES {
        let (reference, required) = commit(entry, FUNDED_WORK_CEILING, true, false);
        let reference = reference.unwrap();
        assert_eq!(
            reference.changed_records, ordinary.changed_records,
            "{entry:?}: same mutation result"
        );
        assert_eq!(
            reference.version_id, ordinary.version_id,
            "{entry:?}: same successor version"
        );
        assert!(required > 0 && required < FUNDED_WORK_CEILING);
    }
}

#[test]
fn a_cancelled_authoritative_commit_keeps_its_typed_stop_and_head() {
    for entry in ENTRIES {
        let (outcome, _) = commit(entry, FUNDED_WORK_CEILING, false, true);
        assert_cancelled(outcome);
    }
    let (outcome, _) = commit(
        Entry::CancelledAfterValidation,
        FUNDED_WORK_CEILING,
        false,
        false,
    );
    assert_cancelled(outcome);
}

fn assert_cancelled(outcome: Result<CommitResult, TransactionCommitError>) {
    let TransactionCommitError::Execution { denial, .. } = outcome.unwrap_err() else {
        panic!("an authoritative commit retains the request stop");
    };
    assert_eq!(
        denial.kind,
        CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::Cancelled),
    );
}
