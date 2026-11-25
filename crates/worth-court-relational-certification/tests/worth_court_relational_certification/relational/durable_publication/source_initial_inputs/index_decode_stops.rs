//! The invoking request stops at the selected original index DATA decoder.
use super::selected_source::{MemberStop, SelectedSource};
use super::*;
use std::sync::{atomic::Ordering, Mutex};
use worth_execution::{
    CancellationSource, ExecutionRequestArrayBuilder, SerialMemoryBudget, SerialRequest,
};
use worth_relational::facade::{indexes::*, transactions::RelationalExecutionDenialCause};

#[test]
fn original_index_decode_refuses_cancelled_and_exhausted_invoking_requests_without_retaining_data()
{
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&policy);
    let (runtime, source, member, members) = recovered_indexes(&setup);
    drop(capture_indexes(&runtime, &source, &members, &setup).unwrap());

    let cancellation = CancellationSource::new();
    let budget = SerialMemoryBudget::new(None);
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    source.arm(member, MemberStop::Cancel(cancellation));
    assert!(matches!(
        capture_indexes(
            &runtime,
            &source,
            &members,
            &ExecutionRequest::serial(&host)
        ),
        Err(RelationalNativeCheckpointCaptureDenial::Request(
            RelationalExecutionDenialCause::Cancelled
        ))
    ));
    assert!(
        source.fired.load(Ordering::Relaxed),
        "the original index member was read before decoder cancellation"
    );
    assert_eq!(budget.charged_memory_bytes(), 0);

    let limit = 64 * 1024 * 1024;
    let budget = SerialMemoryBudget::new(Some(limit));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let held = Arc::new(Mutex::new(None));
    source.arm(member, MemberStop::Exhaust { held: held.clone() });
    assert!(matches!(
        capture_indexes(
            &runtime,
            &source,
            &members,
            &ExecutionRequest::serial(&host)
        ),
        Err(RelationalNativeCheckpointCaptureDenial::Request(
            RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
        ))
    ));
    assert!(
        source.fired.load(Ordering::Relaxed),
        "the original index member was read before decoder exhaustion"
    );
    // Release only the deliberate blocker. No failed invocation DATA remains.
    drop(
        held.lock()
            .unwrap()
            .take()
            .expect("the invoking original request funded the blocker"),
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
    let success = capture_indexes(&runtime, &source, &members, &setup).unwrap();
    assert_eq!(success.source_artifact().covered().0, 1);
}

fn capture_indexes(
    runtime: &RelationalRuntime,
    source: &Arc<SelectedSource>,
    members: &[RelationalDurableMemberLocator],
    request: &ExecutionRequest<'_, '_>,
) -> Result<CompleteRelationalNativeCheckpoint, RelationalNativeCheckpointCaptureDenial> {
    runtime.complete_checkpoint_service().prepare(
        request,
        RelationalCheckpointArtifactPolicy::ExternalMembers {
            current_members: members,
            source: Some(source.medium.clone()),
        },
    )
}

fn recovered_indexes(
    request: &ExecutionRequest<'_, '_>,
) -> (
    RelationalRuntime,
    Arc<SelectedSource>,
    RelationalDurableMemberLocator,
    worth_execution::ExecutionArray<RelationalDurableMemberLocator>,
) {
    let (runtime, medium) = initialized(request);
    let definition = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "original-decoder-stop".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: worth_relational::facade::transactions::planned_single_field_locator(
                    worth_foundational::facade::AspectKey::new("name").unwrap(),
                    worth_foundational::facade::FieldKey::new("name").unwrap(),
                ),
            },
        })
        .unwrap();
    let commit = create_entity_outcome(request, &runtime, "decoder-original");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let built = runtime.index_authority().build_for_basis(
        request,
        DerivedIndexBuildRequest {
            source_commit_id: commit.commit.commit_id,
            branch_id: commit.commit.branch_id.clone(),
            index_ids: vec![definition.index_id],
        },
        &basis,
    );
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    assert!(!built.generations.is_empty());
    drop(basis);
    release_test_commit_snapshot(&runtime, &commit);
    let image = capture_external(&runtime, &medium, request);
    let members = selected_members(&image, request);
    medium.file_checkpoint(image.checkpoint().clone());
    let payload = image
        .checkpoint()
        .bytes()
        .strip_prefix(b"WORTH-REL-CHK23\n")
        .unwrap();
    let member = rmp_serde::from_slice(crate::messagepack_value_spans::field(
        crate::messagepack_value_spans::field(payload, "manifest"),
        "derived_indexes",
    ))
    .unwrap();
    let observed = SelectedSource::new(medium.clone());
    let source: Arc<dyn RelationalDurableColdSource> = observed.clone();
    let cut = RelationalDurableReadProtection::capture(&source, request).unwrap();
    let initial = cut
        .read_initial_inputs_owned(&source, request, 1 << 30)
        .unwrap();
    let input = RelationalDurableRecoverySource::from_home_original_bytes(
        Some(home_image_checkpoint(&image, image.source_artifact().clone(), request).unwrap()),
        initial,
        ExecutionRequestArrayBuilder::allocate(*request, 0)
            .unwrap()
            .seal()
            .unwrap(),
        ExecutionRequestArrayBuilder::allocate(*request, 0)
            .unwrap()
            .seal()
            .unwrap(),
        cut.bounds(),
        source,
        None,
        request,
    )
    .unwrap();
    let mut recovered = receiver(&medium);
    let plan = recovered
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    recovered
        .durability_recovery()
        .recover_with_authority(request, plan)
        .unwrap();
    (recovered, observed, member, members)
}

#[test]
fn complete_fold_matches_original_relation_delete_with_snapshot_and_replay_pins() {
    use super::super::native_checkpoint_partitions::{
        original as capture_original, partition_image, physical_index,
    };
    use worth_relational::facade::{
        identity::{KindId, PartitionId},
        storage::RecordLifecycleState,
        transactions::*,
    };
    use worth_testkit_relational::{
        aspect_storage::publications::begin_main_transaction,
        durable_publication::{field_patch, memory_runtime},
        ordinary_records::create_entity,
    };
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(16 * 1024 * 1024)),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let source = create_entity(&execution, &runtime, "complete-pin-left");
    let target = create_entity(&execution, &runtime, "complete-pin-right");
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("actual pinned relation").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: "complete-pinned-edge".into(),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                    fields: field_patch("label", "label", "complete-pinned-edge"),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let created = transaction.commit(&execution, &runtime).unwrap();
    let [RecordRef::Relation(relation)] = created.changed_records.as_slice() else {
        panic!("the performed creation issued one relation");
    };
    let relation = *relation;
    let old_root = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let old_axes = old_root.reference().target().clone();
    let old_checkpoint = capture_original(&execution, &runtime);
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
    assert!(creation_basis
        .custody_witness()
        .same_root(&old_root.custody_witness()));
    assert_eq!(creation_basis.reference().target(), &old_axes);
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
    let mut transaction = begin_main_transaction(&runtime);
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
    let identity = runtime.main_branch_identity();
    let original = runtime.admit_branch_basis(&identity).unwrap();
    let original_axes = original.reference().target().clone();
    let ordinary = capture_original(&execution, &runtime);
    let current = partition_image(&ordinary, deleted.commit.commit_id, relation.partition_id);
    let arena = current.relation_arena;
    let physical = physical_index(&arena.slots, relation.local_slot_value());
    let lifecycle = arena.lifecycle[physical];
    assert_eq!(lifecycle, RecordLifecycleState::DeletedRetained);
    assert_eq!(arena.snapshot_pins.get(physical), Some(&0));
    assert_eq!(arena.replay_pins.get(physical), Some(&0));
    let old = partition_image(
        &old_checkpoint,
        created.commit.commit_id,
        relation.partition_id,
    );
    let old_physical = physical_index(&old.relation_arena.slots, relation.local_slot_value());
    assert_eq!(
        old.relation_arena.lifecycle[old_physical],
        RecordLifecycleState::Live
    );
    // This invokes the actual public weak service and genuine all-prefix fold.
    // No ordinary checkpoint is promoted and no expected root is injected.
    let complete = runtime
        .complete_checkpoint_service()
        .prepare(&execution, RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None })
        .unwrap();
    let image = partition_image(
        complete.checkpoint(),
        deleted.commit.commit_id,
        relation.partition_id,
    );
    assert_eq!(image.relation_arena.lifecycle[physical], lifecycle);
    assert_eq!(
        image.relation_arena.retired_at[physical],
        Some(deleted.version_id)
    );
    let retained = partition_image(
        complete.checkpoint(),
        created.commit.commit_id,
        relation.partition_id,
    );
    assert_eq!(
        retained.relation_arena.lifecycle[old_physical],
        RecordLifecycleState::Live
    );
    let after = runtime.admit_branch_basis(&identity).unwrap();
    assert_eq!(after.reference().target(), &original_axes);
    assert!(after
        .custody_witness()
        .same_root(&original.custody_witness()));
    let unchanged = capture_original(&execution, &runtime);
    assert_eq!(
        partition_image(&unchanged, deleted.commit.commit_id, relation.partition_id)
            .relation_arena
            .lifecycle[physical],
        lifecycle
    );
    runtime.release_component_basis(external).unwrap();
    assert!(runtime
        .history_authority()
        .release_version_replay_retention(created.version_id));
    release_test_commit_snapshot(&runtime, &created);
    release_test_commit_snapshot(&runtime, &deleted);
}
