//! Real mirror pins cannot change the immutable root's exact deletion lifecycle.
use crate::durability::data::RelationalCheckpointArtifactPolicy;
use crate::storage::data::RecordLifecycleState;
use crate::tests::support::*;
use crate::transactions::data::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

#[test]
fn complete_fold_matches_original_relation_delete_with_snapshot_and_replay_pins() {
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(16 * 1024 * 1024)),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let source = create_entity(&execution, &runtime, "complete-pin-left");
    let target = create_entity(&execution, &runtime, "complete-pin-right");
    let created =
        create_relation_outcome(&execution, &runtime, source, target, "complete-pinned-edge");
    let relation = changed_relations(&created)[0];
    let old_root = runtime
        .history
        .branch_cell(runtime.main_branch_identity().branch_id())
        .unwrap()
        .root()
        .unwrap();
    // Snapshot readability alone is not an external durable component.
    // Retain the owner's exact creation basis before advancing the main head.
    let (_, creation_basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert_eq!(
        creation_basis
            .reference()
            .target()
            .as_basis()
            .unwrap()
            .selected_commit_id(),
        created.commit.commit_id.0,
    );
    assert!(std::sync::Arc::ptr_eq(
        &creation_basis.inner.root,
        &old_root
    ));
    assert_eq!(creation_basis.inner.root.axes(), old_root.axes());
    let external = runtime.retain_component_basis(&creation_basis).unwrap();
    assert_eq!(external.descriptor(), creation_basis.descriptor());
    drop(creation_basis);
    // Keep the actual creation snapshot live, and acquire a real replay pin
    // BEFORE deletion through the existing owner port, not a raw counter edit.
    assert!(runtime
        .history_authority()
        .retain_version_for_replay(&execution, created.version_id)
        .unwrap());
    let before = runtime
        .inspect_what_happened()
        .inspect_record_retention(&execution, RecordRef::Relation(relation))
        .unwrap()
        .unwrap();
    assert!(runtime
        .inspect_what_happened()
        .inspect_snapshot_pinning(&execution, &created.snapshot)
        .unwrap()
        .is_some());
    assert!(before.pins.replay_pins > 0);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("complete-pinned-delete").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: relation,
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let deleted = transaction.commit(&execution, &runtime).unwrap();
    let pinned = runtime
        .inspect_what_happened()
        .inspect_record_retention(&execution, RecordRef::Relation(relation))
        .unwrap()
        .unwrap();
    assert!(pinned.pins.replay_pins > 0);
    let snapshot = runtime
        .read_truth()
        .read_snapshot(&created.snapshot, &execution)
        .unwrap()
        .unwrap();
    let old_relation = snapshot
        .relations()
        .iter()
        .find(|record| record.relation_id == relation)
        .unwrap();
    assert_eq!(old_relation.lifecycle, RecordLifecycleState::Live);
    assert_eq!((old_relation.source, old_relation.target), (source, target));
    drop(snapshot);
    let original = runtime
        .history
        .branch_cell(runtime.main_branch_identity().branch_id())
        .unwrap()
        .root()
        .unwrap();
    let original_axes = original.axes();
    let lease = original
        .resident_full_region_witness(relation.partition_id)
        .unwrap()
        .unwrap();
    let full = lease.full_view();
    let arena = &full.partition().relation_arena;
    let physical = arena.physical_index(relation.slot_index()).unwrap();
    let lifecycle = arena.lifecycle[physical];
    assert_eq!(lifecycle, RecordLifecycleState::DeletedRetained);
    assert_eq!(arena.snapshot_pin_count(relation.slot_index()), Some(0));
    assert_eq!(arena.replay_pin_count(relation.slot_index()), Some(0));
    let old_lease = old_root
        .resident_full_region_witness(relation.partition_id)
        .unwrap()
        .unwrap();
    let old_full = old_lease.full_view();
    let old_arena = &old_full.partition().relation_arena;
    let old_physical = old_arena.physical_index(relation.slot_index()).unwrap();
    assert_eq!(
        old_arena.lifecycle[old_physical],
        RecordLifecycleState::Live
    );
    // This invokes the actual public weak service and genuine all-prefix fold.
    // No ordinary checkpoint is promoted and no expected root is injected.
    let complete = runtime
        .complete_checkpoint_service()
        .prepare(&execution, RelationalCheckpointArtifactPolicy::Embedded)
        .unwrap();
    let decoded = crate::durability::log::native_checkpoint_codec::decode_checkpoint(
        &execution,
        complete.checkpoint().bytes(),
    )
    .unwrap()
    .checkpoint;
    let image = decoded
        .branch_roots
        .iter()
        .find(|image| image.commit_id == deleted.commit.commit_id)
        .expect("actual current root was emitted");
    let image = image
        .partition_images
        .iter()
        .find(|image| image.partition_id == relation.partition_id)
        .unwrap();
    assert_eq!(image.relation_arena.lifecycle[physical], lifecycle);
    assert_eq!(
        image.relation_arena.retired_at[physical],
        Some(deleted.version_id)
    );
    let retained = decoded
        .branch_roots
        .iter()
        .find(|image| image.commit_id == created.commit.commit_id)
        .expect("actual externally retained creation root was emitted");
    let retained = retained
        .partition_images
        .iter()
        .find(|image| image.partition_id == relation.partition_id)
        .unwrap();
    assert_eq!(
        retained.relation_arena.lifecycle[old_physical],
        RecordLifecycleState::Live
    );
    assert_eq!(original.axes(), original_axes);
    assert_eq!(arena.lifecycle[physical], lifecycle);
    runtime.release_component_basis(external).unwrap();
    assert!(runtime
        .history_authority()
        .release_version_replay_retention(created.version_id));
    release_test_commit_snapshot(&runtime, &created);
    release_test_commit_snapshot(&runtime, &deleted);
}
