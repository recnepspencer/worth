//! Records committed through ordinary admission for proposal projection courts.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId, RelationId},
    runtime::RelationalRuntime,
    transactions::{
        AspectFieldPatch, CommitResult, CreateIntent, EntityMutationIntent, EntityReference,
        EntitySpec, MutationIntent, RecordRef, RelationSpec, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::publications::{begin_main_transaction, release_test_commit_snapshot},
    durable_publication::field_patch,
};

pub(super) fn name_field_patch(name: &str) -> AspectFieldPatch {
    field_patch("name", "name", name)
}

pub(super) fn create_entity_with_fields(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    fields: AspectFieldPatch,
) -> EntityId {
    match create(
        execution,
        runtime,
        key,
        CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: key.into(),
            fields,
        }),
    ) {
        RecordRef::Entity(id) => id,
        other => panic!("expected the authored entity, got {other:?}"),
    }
}

pub(super) fn create_relation(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    key: &str,
) -> RelationId {
    create_relation_with_fields(
        execution,
        runtime,
        source,
        target,
        key,
        field_patch("label", "label", key),
    )
}

pub(super) fn create_relation_with_fields(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    key: &str,
    fields: AspectFieldPatch,
) -> RelationId {
    match create(
        execution,
        runtime,
        key,
        CreateIntent::Relation(RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: key.into(),
            source: EntityReference::Existing(source),
            target: EntityReference::Existing(target),
            fields,
        }),
    ) {
        RecordRef::Relation(id) => id,
        other => panic!("expected the authored relation, got {other:?}"),
    }
}

pub(super) fn create(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    intent: CreateIntent,
) -> RecordRef {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new(key).push(MutationIntent::Create(intent)),
            execution.allocation_policy(),
        )
        .unwrap();
    let result = transaction.commit(execution, runtime).unwrap();
    let [record] = result.changed_records.as_slice() else {
        panic!("the fixture creates one record");
    };
    release_test_commit_snapshot(runtime, &result);
    record.clone()
}

pub(super) fn update_name(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: EntityId,
    value: &str,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("update-entity").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id,
                    fields: field_patch("name", "name", value),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    transaction.commit(execution, runtime).unwrap()
}

pub(super) fn create_relation_in_partition(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    key: &str,
    partition_id: PartitionId,
) -> RelationId {
    let RecordRef::Relation(relation) = create(
        execution,
        runtime,
        key,
        CreateIntent::Relation(RelationSpec {
            partition_id,
            kind_id: KindId(2),
            client_key: key.into(),
            source: EntityReference::Existing(source),
            target: EntityReference::Existing(target),
            fields: field_patch("label", "label", key),
        }),
    ) else {
        panic!("expected the authored relation");
    };
    relation
}
