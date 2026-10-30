use crate::transactions::data::{
    CommitExecutionDenialKind, CommitResult, TransactionCommitError, WorkerIntentBatch,
};
use worth_execution::{CancellationSource, ExecutionRequest, ExecutionWorkCeiling, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[derive(Clone, Copy, Debug)]
enum Entry {
    Ordinary,
    Batch,
    Split,
    Strategy,
    Authoritative,
    AuthoritativeCancelled,
}
const ENTRIES: [Entry; 5] = [
    Entry::Ordinary,
    Entry::Batch,
    Entry::Split,
    Entry::Strategy,
    Entry::Authoritative,
];

// Fixture funding, not a price for an implementation's incidental dispatches.
const FUNDED_WORK_CEILING: u64 = 1_000_000;

fn commit(
    entry: Entry,
    ceiling: u64,
    observe_whole_request: bool,
    cancel_before_entry: bool,
) -> (Result<CommitResult, TransactionCommitError>, u64) {
    // All entries receive the same lowered mutation: one entity, two fields,
    // the same empty basis and installed invariant families. Only Batch adds
    // two empty batch descriptors; these add no data-work units.
    let (runtime, lowered) = crate::commit_strategies::facade::lowered_commit_fixture();
    let identity = runtime.main_branch_identity();
    let before = runtime.observe_branch(&identity).unwrap().0;
    let cancellation = CancellationSource::new();
    let lease = crate::tests::support::test_execution_authority()
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
        crate::execution::work_observation::capture(|| match entry {
            Entry::Strategy => {
                if cancel_before_entry {
                    cancellation.cancel();
                }
                let mut authority = runtime.commit_strategies_authority();
                let proposal = authority.validate_lowered_plan(&execution, &runtime, lowered)?;
                let candidate =
                    authority.prepare_validated_commit(&execution, &runtime, proposal)?;
                runtime.publish_prepared_candidate(&execution, candidate)
            }
            Entry::Ordinary
            | Entry::Batch
            | Entry::Split
            | Entry::Authoritative
            | Entry::AuthoritativeCancelled => {
                let (mut transaction, _, _, _) = lowered.into_validation_parts();
                if matches!(entry, Entry::Batch) {
                    for label in ["empty-left", "empty-right"] {
                        transaction
                            .push_batch(
                                WorkerIntentBatch::new(label),
                                execution.allocation_policy(),
                            )
                            .unwrap();
                    }
                }
                if cancel_before_entry {
                    cancellation.cancel();
                }
                if matches!(
                    entry,
                    Entry::Split | Entry::Authoritative | Entry::AuthoritativeCancelled
                ) {
                    let proposal = runtime.validate_branch_transaction(&execution, transaction)?;
                    if matches!(entry, Entry::Authoritative | Entry::AuthoritativeCancelled) {
                        if matches!(entry, Entry::AuthoritativeCancelled) {
                            cancellation.cancel();
                        }
                        return crate::authority::commit::pipeline::execute_authoritative_commit(
                        &execution,
                        &runtime,
                        crate::authority::commit::pipeline::AuthoritativeCommitContext::from_validated_proposal(proposal),
                    );
                    }
                    let candidate = runtime.prepare_validated_proposal(&execution, proposal)?;
                    runtime.publish_prepared_candidate(&execution, candidate)
                } else {
                    runtime.commit_branch_transaction(&execution, transaction)
                }
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
fn one_work_account_bounds_the_same_proposal_on_every_entry() {
    // The SDK report includes native evidence work outside the packet recorder.
    let (reference, required_work) = commit(Entry::Ordinary, FUNDED_WORK_CEILING, true, false);
    reference.unwrap();
    assert!(required_work > 0 && required_work < FUNDED_WORK_CEILING);
    let (unwrapped, recorded) = commit(Entry::Ordinary, FUNDED_WORK_CEILING, false, false);
    unwrapped.unwrap();
    assert_eq!(
        recorded, required_work,
        "the authoritative account settles all SDK work"
    );
    let completed = ENTRIES.map(|entry| {
        let (outcome, charged) = commit(entry, required_work, true, false);
        outcome.unwrap();
        charged
    });
    assert_eq!(
        completed,
        [required_work; ENTRIES.len()],
        "the same proposal pays the same full work through every entry"
    );
    // No enclosing account may make an internally reset allowance look sound.
    let outcomes = ENTRIES.map(|entry| commit(entry, required_work - 1, false, false).0);
    let refused = outcomes.each_ref().map(|outcome| {
        matches!(outcome, Err(TransactionCommitError::Execution { denial, .. })
            if denial.kind == CommitExecutionDenialKind::Cause(crate::execution::RelationalExecutionDenialCause::WorkExhausted))
    });
    assert_eq!(
        refused, [true; 5],
        "all entries spend one ceiling below {required_work}"
    );
}

#[test]
fn a_cancelled_authoritative_commit_keeps_its_typed_stop_and_head() {
    for entry in ENTRIES {
        let (outcome, _) = commit(entry, FUNDED_WORK_CEILING, false, true);
        assert_cancelled(outcome);
    }
    let (outcome, _) = commit(
        Entry::AuthoritativeCancelled,
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
        CommitExecutionDenialKind::Cause(
            crate::execution::RelationalExecutionDenialCause::Cancelled
        ),
    );
}
