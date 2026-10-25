//! A test-local counting observer runs immediately before each request consult.
use super::{observation, CommitStage};
use crate::tests::support::runtime_with_test_schema;
use crate::transactions::data::WorkerIntentBatch;
use std::sync::{Arc, Mutex};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

fn assert_one_consult_per_stage(batches: usize, split: bool, strategy: bool) {
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
    // Each literal names the stage beside its single allowed consult.
    let stages = [
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
        PublicationAdmission,
        Publication,
    ];
    let counts = stages.map(|stage| {
        (
            stage,
            visits.iter().filter(|visit| **visit == stage).count(),
        )
    });
    assert_eq!(
        counts,
        [
            (Validation, 1),
            (Admission, 1),
            (Draft, 1),
            (Boundary, 1),
            (Mutation, 1),
            (History, 1),
            (Snapshot, 1),
            (Artifact, 1),
            (PublicationPreparation, 1),
            (CandidateRegistration, 1),
            (PublicationAdmission, 1),
            (Publication, 1)
        ]
    );
    assert_eq!(*visits, stages);
}

#[test]
fn ordinary_commit_consults_once_per_named_stage() {
    assert_one_consult_per_stage(1, false, false);
}
#[test]
fn batch_commit_consults_once_per_named_stage() {
    assert_one_consult_per_stage(3, false, false);
}

#[test]
fn split_commit_consults_once_per_named_stage() {
    assert_one_consult_per_stage(1, true, false);
}

#[test]
fn strategy_commit_consults_once_per_named_stage() {
    assert_one_consult_per_stage(1, false, true);
}
