//! An exact unique index avoids discovering untouched remote entity slots.
use super::{
    candidate_source::recovered,
    current_field_meanings::*,
};
use std::sync::atomic::Ordering;
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    symbols::ClientKey,
    transactions::*,
};
use worth_testkit_relational::aspect_storage::publications::{
    begin_main_transaction, changed_entities, release_test_commit_snapshot,
};
#[derive(Clone, Copy, Debug)]
pub(super) enum PreparationDoor {
    Validation,
    Candidate,
    Commit,
}
pub(super) fn transaction(
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

pub(super) fn storage(
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

impl PreparationDoor {
    pub(super) const ALL: [Self; 3] = [Self::Validation, Self::Candidate, Self::Commit];
    pub(super) fn run(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<(), worth_relational::facade::storage::RelationalStorageUnavailable> {
        let transaction = transaction(request, runtime);
        match self {
            Self::Validation => transaction.validate(request, runtime).map(|_| ()),
            Self::Candidate => runtime
                .prepare_branch_transaction(request, transaction)
                .map(|_| ()),
            Self::Commit => transaction.commit(request, runtime).map(|_| ()),
        }
        .map_err(storage)
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
    let before_cold = cold.runtime.observe_complexity_counters();
    let before_warm = warm.runtime.observe_complexity_counters();
    let cold_commit = transaction(request, &cold.runtime)
        .commit(request, &cold.runtime)
        .unwrap();
    let warm_commit = transaction(request, &warm.runtime)
        .commit(request, &warm.runtime)
        .unwrap();
    let cold_entity = changed_entities(&cold_commit)[0];
    let warm_entity = changed_entities(&warm_commit)[0];
    assert_eq!(
        cold_entity.slot_index(), warm_entity.slot_index(),
        "the exact index avoids cold record discovery and abandoned identities"
    );
    assert!((cold.floor..cold.floor + 16).contains(&cold_entity.slot_index()));
    assert!((warm.floor..warm.floor + 16).contains(&warm_entity.slot_index()));
    let after_cold = cold.runtime.observe_complexity_counters();
    let after_warm = warm.runtime.observe_complexity_counters();
    assert!(after_cold.preparation_packet_count > before_cold.preparation_packet_count);
    assert_eq!(
        after_cold.preparation_packet_count - before_cold.preparation_packet_count,
        after_warm.preparation_packet_count - before_warm.preparation_packet_count
    );
    assert_eq!(
        after_cold.invariant_entity_slot_scans - before_cold.invariant_entity_slot_scans,
        after_warm.invariant_entity_slot_scans - before_warm.invariant_entity_slot_scans
    );
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
