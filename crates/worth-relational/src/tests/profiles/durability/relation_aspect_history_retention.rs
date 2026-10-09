use crate::facade::config::{MvccConfig, RetentionBackend, SnapshotReleasePolicy};
use crate::tests::support::*;

#[test]
fn retained_root_preserves_relation_aspect_history_without_record_pins() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .mvcc(MvccConfig {
            track_visibility_metadata: true,
            snapshot_release_policy: SnapshotReleasePolicy::ExplicitRelease,
            auto_reclaim_deleted_records: true,
            reclaim_batch_size: 32,
            retention_backend: RetentionBackend::PinTrackedRetention,
        })
        .build();
    let source_outcome = create_entity_outcome(&runtime, "source");
    let target_outcome = create_entity_outcome(&runtime, "target");
    let source = changed_entities(&source_outcome)[0];
    let target = changed_entities(&target_outcome)[0];
    let relation = create_aspect_bearing_relation(&runtime, source, target);
    let identity = runtime.main_branch_identity();
    let (_, retained_basis) = runtime.observe_branch(&identity).unwrap();

    let deleted = delete_relation(&runtime, relation);
    runtime
        .visibility_authority()
        .release_snapshot(&source_outcome.snapshot)
        .unwrap();
    runtime
        .visibility_authority()
        .release_snapshot(&target_outcome.snapshot)
        .unwrap();
    runtime
        .visibility_authority()
        .release_snapshot(&deleted.created_snapshot)
        .unwrap();
    runtime
        .visibility_authority()
        .release_snapshot(&deleted.deleted_snapshot)
        .unwrap();
    let plan = runtime.retention().inspect_plan();
    assert_eq!(plan.snapshot_pinned_relations, 0);
    assert_eq!(plan.branch_pinned_relations, 0);
    let maintenance = runtime.retention().run_pass();
    assert!(maintenance.relation_reclaimed <= 1);

    assert_eq!(runtime.relation_history_len_for_test(relation), 1);
    let historical = runtime
        .read_truth()
        .read_observation(&retained_basis.observation())
        .unwrap();
    let relation_record = historical
        .relations
        .iter()
        .find(|record| record.relation_id == relation)
        .expect("retained relation aspect record");
    assert_eq!(
        read_relation_field(relation_record, field_key("label")),
        Some("r1".into())
    );
}

struct DeletedAspectRelationEvidence {
    created_snapshot: crate::snapshots::data::SnapshotHandle,
    deleted_snapshot: crate::snapshots::data::SnapshotHandle,
}

fn create_aspect_bearing_relation(
    runtime: &RelationalRuntime,
    source: crate::facade::identity::EntityId,
    target: crate::facade::identity::EntityId,
) -> crate::facade::identity::RelationId {
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(
        WorkerIntentBatch::new("aspect-bearing-relation").push(MutationIntent::Create(
            CreateIntent::Relation(crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("r1"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: relation_label_field_patch("r1"),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let created = txn
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let relation = changed_relations(&created)[0];
    assert_eq!(runtime.relation_history_len_for_test(relation), 1);
    release_test_commit_snapshot(runtime, &created);
    relation
}

fn delete_relation(
    runtime: &RelationalRuntime,
    relation: crate::facade::identity::RelationId,
) -> DeletedAspectRelationEvidence {
    let created_snapshot = runtime.visibility_authority().snapshot();
    let mut delete_txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    delete_txn
        .push_batch(
            WorkerIntentBatch::new("delete-relation").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: relation,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let deleted = delete_txn
        .commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    DeletedAspectRelationEvidence {
        created_snapshot,
        deleted_snapshot: deleted.snapshot.clone(),
    }
}
