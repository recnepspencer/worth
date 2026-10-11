use crate::transactions::data::{
    CommitExecutionDenialKind, CommitResult, TransactionCommitError, WorkerIntentBatch,
};
use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[derive(Clone, Copy, Debug)]
enum Entry {
    Ordinary,
    Batch,
    Split,
    Strategy,
}
const ENTRIES: [Entry; 4] = [Entry::Ordinary, Entry::Batch, Entry::Split, Entry::Strategy];

fn commit(entry: Entry, ceiling: u64) -> (Result<CommitResult, TransactionCommitError>, u64) {
    // All entries receive the same lowered mutation: one entity, two fields,
    // the same empty basis and installed invariant families. Only Batch adds
    // two empty batch descriptors; these add no data-work units.
    let (runtime, lowered) = crate::commit_strategies::facade::lowered_commit_fixture();
    let identity = runtime.main_branch_identity();
    let before = runtime.observe_branch(&identity).unwrap().0;
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::MIN, 64 * 1024 * 1024, ceiling),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
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
        Entry::Ordinary | Entry::Batch | Entry::Split => {
            let (mut transaction, _, _, _) = lowered.into_validation_parts();
            if matches!(entry, Entry::Batch) {
                for label in ["empty-left", "empty-right"] {
                    transaction
                        .push_batch(WorkerIntentBatch::new(label), execution.allocation_policy())
                        .unwrap();
                }
            }
            if matches!(entry, Entry::Split) {
                let proposal = runtime.validate_branch_transaction(&execution, transaction)?;
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
        let (outcome, charged) = commit(entry, 116);
        outcome.unwrap();
        charged
    });
    assert_eq!(
        completed, [116; 4],
        "one entity with two fields: ordinary, batch, split, strategy each charge 116"
    );
    let refused = ENTRIES.map(|entry| {
        let (outcome, _) = commit(entry, 115);
        matches!(outcome, Err(TransactionCommitError::Execution { denial, .. })
            if denial.kind == CommitExecutionDenialKind::Cause(crate::execution::RelationalExecutionDenialCause::WorkExhausted))
    });
    assert_eq!(
        refused, [true; 4],
        "ordinary, batch, split, strategy all spend one ceiling"
    );
}
