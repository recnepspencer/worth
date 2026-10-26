//! Prepared resident targets preserve live metadata and private new partitions.
use crate::mvcc::{PreparedRelationalCommitCandidate, RelationalPublicationOutcome};
use crate::storage::substrate::{EntityRecordKind, PinClass};
use crate::tests::support::*;
use crate::transactions::data::{EntityMutationIntent, RecordRef, UpdateEntityFieldsIntent};

#[test]
fn a_new_partition_target_stays_private_until_its_candidate_is_published() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let partition = PartitionId(53);
    let frontier = runtime.history().latest_patch_stream_position();
    let create = || {
        let mut batch = batch_create("private-target");
        let MutationIntent::Create(CreateIntent::Entity(entity)) = &mut batch.intents[0] else {
            unreachable!("the producer declares an entity create");
        };
        entity.partition_id = partition;
        prepare(execution, &runtime, batch)
    };
    let abandoned = create();
    let abandoned_id = changed_entity(&abandoned);
    assert!(!runtime
        .storage_access()
        .partition_ids()
        .contains(&partition));
    assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
    drop(abandoned);
    assert!(!runtime
        .storage_access()
        .partition_ids()
        .contains(&partition));

    let accepted = create();
    let accepted_id = changed_entity(&accepted);
    assert_ne!(
        accepted_id, abandoned_id,
        "abandoning a private target spends its identity"
    );
    assert!(!runtime
        .storage_access()
        .partition_ids()
        .contains(&partition));
    let outcome = publish(execution, &runtime, accepted);
    assert!(runtime
        .storage_access()
        .partition_ids()
        .contains(&partition));
    assert_eq!(changed_entities(&outcome), vec![accepted_id]);
    assert!(runtime
        .read_truth()
        .visible_entity_at_version(execution, accepted_id, runtime.current_version_id(),)
        .unwrap()
        .is_some());
    release_test_commit_snapshot(&runtime, &outcome);
}

#[test]
fn publication_edits_the_latest_target_pins_and_preserves_earlier_editions() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let before_commit = create_entity_outcome(execution, &runtime, "target-before");
    let entity = changed_entities(&before_commit)[0];
    let before_record = runtime
        .read_truth()
        .visible_entity_at_version(execution, entity, before_commit.version_id)
        .unwrap()
        .unwrap();
    let old_edition = runtime.acquire_partition_edition();
    let witness = old_edition.resident_pin_lease(entity.partition_id).unwrap();
    let candidate = prepare(
        execution,
        &runtime,
        WorkerIntentBatch::new("target-after").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: name_field_patch("target-after"),
            }),
        )),
    );
    assert_eq!(
        old_edition
            .partition(entity.partition_id)
            .unwrap()
            .entity_arena
            .replay_pin_count(entity.slot_index()),
        Some(0)
    );
    runtime
        .storage_authority()
        .adjust_named_pin::<EntityRecordKind>(
            &witness,
            entity.slot_index(),
            PinClass::Replay,
            1,
            runtime.current_version_id(),
        );
    let pinned = runtime.acquire_partition_edition();
    assert_eq!(
        pinned
            .partition(entity.partition_id)
            .unwrap()
            .entity_arena
            .replay_pin_count(entity.slot_index()),
        Some(1)
    );
    let outcome = publish(execution, &runtime, candidate);
    let current = runtime.acquire_partition_edition();
    assert_eq!(
        current
            .partition(entity.partition_id)
            .unwrap()
            .entity_arena
            .replay_pin_count(entity.slot_index()),
        Some(1)
    );
    assert_eq!(
        old_edition
            .partition(entity.partition_id)
            .unwrap()
            .entity_arena
            .replay_pin_count(entity.slot_index()),
        Some(0)
    );
    assert_eq!(
        pinned
            .partition(entity.partition_id)
            .unwrap()
            .entity_arena
            .replay_pin_count(entity.slot_index()),
        Some(1)
    );
    let retained = runtime
        .read_truth()
        .read_snapshot(&before_commit.snapshot, execution)
        .unwrap()
        .unwrap();
    assert_eq!(retained.entities, vec![before_record]);
    runtime
        .storage_authority()
        .adjust_named_pin::<EntityRecordKind>(
            &witness,
            entity.slot_index(),
            PinClass::Replay,
            -1,
            runtime.current_version_id(),
        );
    release_test_commit_snapshot(&runtime, &outcome);
    release_test_commit_snapshot(&runtime, &before_commit);
}

fn prepare(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    batch: WorkerIntentBatch,
) -> PreparedRelationalCommitCandidate {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap()
}

fn changed_entity(
    candidate: &PreparedRelationalCommitCandidate,
) -> crate::identity::data::EntityId {
    candidate
        .with_prepared_changed_records(|records| {
            let [RecordRef::Entity(entity)] = records else {
                panic!("producer changes one entity");
            };
            *entity
        })
        .unwrap()
}

fn publish(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    candidate: PreparedRelationalCommitCandidate,
) -> CommitResult {
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the current candidate must publish");
    };
    runtime.settle_performed_publication(performed).unwrap()
}
