//! A test-local counting observer runs immediately before each request consult.
use super::{observation, CommitStage};
use crate::tests::support::runtime_with_test_schema;
use crate::transactions::data::WorkerIntentBatch;
use std::sync::{Arc, Mutex};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

fn assert_one_consult_per_safe_point(batches: usize, split: bool, strategy: bool) {
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    for index in 0..batches {
        transaction
            .push_batch(
                WorkerIntentBatch::new(format!("count-{index}")),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
    }
    let backing = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&backing);
    let visits = Arc::new(Mutex::new(Vec::new()));
    let counted = Arc::clone(&visits);
    let strategy_fixture = strategy.then(crate::commit_strategies::facade::lowered_commit_fixture);
    observation::with_observer(
        Box::new(move |stage| counted.lock().unwrap().push(stage)),
        || {
            if let Some((runtime, lowered)) = strategy_fixture {
                let mut authority = runtime.commit_strategies_authority();
                let proposal = authority.validate_lowered_plan(&execution, &runtime, lowered)?;
                let candidate =
                    authority.prepare_validated_commit(&execution, &runtime, proposal)?;
                runtime.publish_prepared_candidate(&execution, candidate)
            } else if split {
                let proposal = runtime.validate_branch_transaction(&execution, transaction)?;
                let candidate = runtime.prepare_validated_proposal(&execution, proposal)?;
                let outcome = runtime
                    .publication_port()
                    .compare_and_publish(&execution, candidate);
                let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = outcome
                else {
                    panic!("live split publication must perform: {outcome:?}");
                };
                runtime.settle_performed_publication(performed)
            } else {
                runtime.commit_branch_transaction(&execution, transaction)
            }
        },
    )
    .unwrap();
    let visits = visits.lock().unwrap();
    use CommitStage::*;
    // Each literal names one safe point beside its single allowed consult.
    // Publication owns two: its entry, before preflight, and the final
    // pre-effect boundary after preflight, where the caller's original request
    // keeps its first cause ahead of any separate operation control.
    let safe_points = [
        Validation,
        Admission,
        Draft,
        Boundary,
        Mutation,
        History,
        Snapshot,
        Artifact,
        PublicationPreparation,
        CandidateRegistration,
        Publication,
        Publication,
    ];
    assert_eq!(*visits, safe_points);
}

#[test]
fn ordinary_commit_consults_once_per_named_safe_point() {
    assert_one_consult_per_safe_point(1, false, false);
}
#[test]
fn batch_commit_consults_once_per_named_safe_point() {
    assert_one_consult_per_safe_point(3, false, false);
}

#[test]
fn split_commit_consults_once_per_named_safe_point() {
    assert_one_consult_per_safe_point(1, true, false);
}

#[test]
fn strategy_commit_consults_once_per_named_safe_point() {
    assert_one_consult_per_safe_point(1, false, true);
}
