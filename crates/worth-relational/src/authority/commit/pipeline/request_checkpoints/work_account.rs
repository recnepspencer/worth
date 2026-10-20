use crate::transactions::data::{
    CommitExecutionDenialKind, CommitResult, TransactionCommitError, WorkerIntentBatch,
};
use worth_execution::{CancellationSource, ExecutionRequest, LeaseRequest};
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

// The commit-track invoice included one serial-strategy diagnostic per
// validation point. The edges width law prices that metadata at zero while
// retaining its liveness consult and storage claim. Adjust only that row of
// the existing invoice; no observed charge supplies this quote.
const INVOICE_WITH_DIAGNOSTIC_WORK: u64 = 116;
const VALIDATION_POINTS: u64 = 3;
const FORMER_DIAGNOSTIC_PRICE: u64 = 1;
const DIAGNOSTIC_PRICE: u64 = 0;
const COMMIT_WORK: u64 =
    INVOICE_WITH_DIAGNOSTIC_WORK - VALIDATION_POINTS * (FORMER_DIAGNOSTIC_PRICE - DIAGNOSTIC_PRICE);

fn commit(entry: Entry, ceiling: u64) -> (Result<CommitResult, TransactionCommitError>, u64) {
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
    let (outcome, charges) = crate::execution::work_observation::capture(|| match entry {
        Entry::Strategy => {
            let mut authority = runtime.commit_strategies_authority();
            let proposal = authority.validate_lowered_plan(&execution, &runtime, lowered)?;
            let candidate = authority.prepare_validated_commit(&execution, &runtime, proposal)?;
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
                        .push_batch(WorkerIntentBatch::new(label), execution.allocation_policy())
                        .unwrap();
                }
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
    });
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
    (outcome, charges.into_iter().sum())
}

#[test]
fn one_work_account_bounds_the_same_proposal_on_every_entry() {
    // Sum dispatch reports, rather than reading the account's stored balance.
    // No enclosing scope is allowed to hide the split-account defect.
    let completed = ENTRIES.map(|entry| {
        let (outcome, charged) = commit(entry, COMMIT_WORK);
        outcome.unwrap();
        charged
    });
    assert_eq!(
        completed,
        [COMMIT_WORK; ENTRIES.len()],
        "one entity with two fields: every commit entry pays the same exact invoice"
    );
    let refused = ENTRIES.map(|entry| {
        let (outcome, _) = commit(entry, COMMIT_WORK - 1);
        matches!(outcome, Err(TransactionCommitError::Execution { denial, .. })
            if denial.kind == CommitExecutionDenialKind::Cause(crate::execution::RelationalExecutionDenialCause::WorkExhausted))
    });
    assert_eq!(
        refused, [true; 5],
        "ordinary, batch, split, strategy and authoritative all spend one ceiling"
    );
}

#[test]
fn a_cancelled_authoritative_commit_keeps_its_typed_stop_and_head() {
    let (outcome, _) = commit(Entry::AuthoritativeCancelled, COMMIT_WORK);
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
