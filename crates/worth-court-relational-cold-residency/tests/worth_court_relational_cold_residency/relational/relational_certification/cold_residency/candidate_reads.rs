//! Validation, candidate and commit owners preserve the same remote read refusal.
use super::{
    candidate_source::{recovered, REMOTE},
    current_field_meanings::*,
};
use std::sync::atomic::Ordering;
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    storage::RelationalStorageUnavailableCause as Cause,
    symbols::ClientKey,
    transactions::*,
};
use worth_testkit_relational::aspect_storage::publications::{
    begin_main_transaction, changed_entities, release_test_commit_snapshot,
};
#[derive(Clone, Copy, Debug)]
enum PreparationDoor {
    Validation,
    Candidate,
    Commit,
}
fn transaction(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
) -> worth_relational::facade::mvcc::BranchBoundRelationalTransaction {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("candidate-probe").push(MutationIntent::Create(
                CreateIntent::Entity(worth_relational::facade::transactions::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("candidate-value"),
                    fields: name_field_patch("candidate-value"),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    transaction
}

fn storage(
    error: TransactionCommitError,
) -> worth_relational::facade::storage::RelationalStorageUnavailable {
    let TransactionCommitError::Preparation { error, .. } = error else {
        panic!("candidate storage failure is pre-effect: {error:?}");
    };
    let CommitPreparationReason::StorageUnavailable(failure) = error.reason() else {
        panic!("storage refusal retains its native payload: {error:?}");
    };
    failure
}

#[test]
fn candidate_invariant_discovery_refuses_stickily_without_publication() {
    let backing = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&backing);
    for door in [
        PreparationDoor::Validation,
        PreparationDoor::Candidate,
        PreparationDoor::Commit,
    ] {
        let fixture = recovered(execution);
        let runtime = fixture.runtime;
        let source = fixture.source;
        let medium = fixture.medium;
        let diagnostics = runtime.publication().diagnostics_snapshot().diagnostics;
        let frontier = runtime.history().latest_patch_stream_position();
        let offered = medium.offered().len();
        source.damaged.store(true, Ordering::Relaxed);
        let run = || {
            let transaction = transaction(execution, &runtime);
            match door {
                PreparationDoor::Validation => {
                    transaction.validate(execution, &runtime).map(|_| ())
                }
                PreparationDoor::Candidate => runtime
                    .prepare_branch_transaction(execution, transaction)
                    .map(|_| ()),
                PreparationDoor::Commit => transaction.commit(execution, &runtime).map(|_| ()),
            }
            .map_err(storage)
        };
        let failure =
            run().expect_err("an untouched cold invariant region must not become absence");
        assert_eq!(
            failure.subject,
            worth_relational::facade::storage::RelationalStorageSubject::Region(REMOTE)
        );
        assert_eq!(failure.cause, Cause::IntegrityMismatch);
        let reads = source.reads.load(Ordering::Relaxed);
        source.damaged.store(false, Ordering::Relaxed);
        assert_eq!(run(), Err(failure));
        assert_eq!(source.reads.load(Ordering::Relaxed), reads);
        assert_eq!(
            runtime.publication().diagnostics_snapshot().diagnostics,
            diagnostics
        );
        assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
        assert_eq!(medium.offered().len(), offered);
    }
}

#[test]
fn cold_and_warm_candidate_preparation_both_publish_the_original_mutation() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    let cold = recovered(request);
    let warm = recovered(request);
    let (_, basis) = warm
        .runtime
        .observe_branch(&warm.runtime.main_branch_identity())
        .unwrap();
    assert!(warm
        .runtime
        .entity_record_at_observation(request, &basis.observation(), warm.remote)
        .unwrap()
        .is_some());
    drop(basis);
    let cold_commit = transaction(request, &cold.runtime)
        .commit(request, &cold.runtime)
        .unwrap();
    let warm_commit = transaction(request, &warm.runtime)
        .commit(request, &warm.runtime)
        .unwrap();
    let cold_entity = changed_entities(&cold_commit)[0];
    let warm_entity = changed_entities(&warm_commit)[0];
    assert!(cold.source.reads.load(Ordering::Relaxed) < 24);
    for (runtime, entity) in [(&cold.runtime, cold_entity), (&warm.runtime, warm_entity)] {
        let mut reader = begin_main_transaction(runtime);
        let record = reader
            .read_entity(request, entity)
            .unwrap()
            .effective()
            .cloned()
            .unwrap();
        assert_eq!(
            super::record_field_values::name(&record),
            Some("candidate-value")
        );
    }
    release_test_commit_snapshot(&cold.runtime, &cold_commit);
    release_test_commit_snapshot(&warm.runtime, &warm_commit);
}
