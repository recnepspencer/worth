use super::fixtures::{
    endpoint_deletion_integrity_declarations, endpoint_kind_integrity_declarations,
    relation_integrity_schema,
};
use crate::facade::storage::RecordLifecycleState;
use crate::tests::support::*;

#[test]
fn rolled_back_illegal_relation_work_leaves_zero_cdc_and_diagnostic_residue() {
    let schema = relation_integrity_schema(
        CascadeDeletePolicy::CascadeDeleteRelations,
        endpoint_kind_integrity_declarations(),
    );
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let checkpoint = checkpoint_for_schema_version(
        runtime.publication().latest_patch().unwrap().position,
        SchemaVersionId(1),
    );

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    let savepoint = txn.create_savepoint().unwrap();
    txn.push_batch(
        WorkerIntentBatch::new("illegal-self-edge").push(MutationIntent::Create(
            CreateIntent::Relation(crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("illegal"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(source),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let rollback = txn.rollback_to_savepoint(savepoint).unwrap();
    txn.push_batch(
        WorkerIntentBatch::new("surviving-edge").push(MutationIntent::Create(
            CreateIntent::Relation(crate::transactions::data::RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw("surviving"),
                source: crate::transactions::data::EntityReference::Existing(source),
                target: crate::transactions::data::EntityReference::Existing(target),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    assert!(rollback.has_effects());
    assert_patch_omits_detail(&outcome, "illegal");
    assert_subscriber_stream_omits_detail(&runtime, checkpoint, "illegal");

    assert!(!runtime
        .publication()
        .diagnostics()
        .artifacts()
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::RelationEndpointKindViolation));
}

#[test]
fn rolled_back_endpoint_deletion_work_leaves_zero_cdc_and_diagnostic_residue() {
    let schema = relation_integrity_schema(
        CascadeDeletePolicy::RetainDanglingForAudit,
        endpoint_deletion_integrity_declarations(),
    );
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .cascade_delete_policy(CascadeDeletePolicy::RetainDanglingForAudit)
        .build();
    let source = create_entity(&runtime, "source");
    let target = create_entity(&runtime, "target");
    let relation_outcome = create_relation_outcome(&runtime, source, target, "live");
    let relation = changed_relations(&relation_outcome)[0];
    let checkpoint = checkpoint_for_schema_version(
        runtime.publication().latest_patch().unwrap().position,
        SchemaVersionId(1),
    );

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    let savepoint = txn.create_savepoint().unwrap();
    txn.push_batch(
        WorkerIntentBatch::new("rolled-back-delete-source").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: source }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let rollback = txn.rollback_to_savepoint(savepoint).unwrap();
    txn.push_batch(
        WorkerIntentBatch::new("surviving-update").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: target,
                fields: single_string_aspect_field_patch(
                    crate::tests::support::aspect_key("name"),
                    field_key("name"),
                    "target-survived",
                ),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();

    assert!(rollback.has_effects());
    assert_patch_omits_detail(&outcome, "RetainedDanglingForAudit");
    assert_subscriber_stream_omits_detail(&runtime, checkpoint, "RetainedDanglingForAudit");

    let read = runtime
        .read_truth()
        .read_snapshot(&outcome.snapshot)
        .unwrap();
    let relation = read.get_relation(relation).unwrap();
    assert_eq!(relation.lifecycle, RecordLifecycleState::Live);
    assert!(!runtime
        .publication()
        .diagnostics()
        .artifacts()
        .iter()
        .flat_map(|artifact| artifact.entries.iter())
        .any(|entry| entry.code == DiagnosticCode::RelationEndpointDeletionIntegrityViolation));
}
