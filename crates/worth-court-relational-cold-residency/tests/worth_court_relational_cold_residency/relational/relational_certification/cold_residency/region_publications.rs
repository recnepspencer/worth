//! Partition-owned graph authoring through ordinary transaction owners.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{identity::*, runtime::RelationalRuntime, transactions::*};
use worth_testkit_relational::{
    aspect_storage::publications::{
        begin_main_transaction, changed_entities, release_test_commit_snapshot,
    },
    durable_publication::field_patch,
};
pub(super) fn create_entity(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    partition_id: PartitionId,
) -> EntityId {
    let mut txn = begin_main_transaction(runtime);
    txn.push_batch(
        WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id,
                kind_id: KindId(1),
                client_key: key.into(),
                fields: field_patch("name", "name", key),
            },
        ))),
        request.allocation_policy(),
    )
    .unwrap();
    let result = txn.commit(request, runtime).unwrap();
    let entity = changed_entities(&result)[0];
    release_test_commit_snapshot(runtime, &result);
    entity
}
pub(super) fn create_relation(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
) {
    let mut txn = begin_main_transaction(runtime);
    txn.push_batch(
        WorkerIntentBatch::new("edge").push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: source.partition_id,
                kind_id: KindId(2),
                client_key: "edge".into(),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: field_patch("label", "label", "edge"),
            },
        ))),
        request.allocation_policy(),
    )
    .unwrap();
    let result = txn.commit(request, runtime).unwrap();
    release_test_commit_snapshot(runtime, &result);
}

/// Publish one actual relation in the explicitly selected owning partition.
pub(super) fn create_relation_in_partition(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    key: &str,
    partition_id: PartitionId,
) -> RelationId {
    let mut txn = begin_main_transaction(runtime);
    txn.push_batch(
        WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id,
                kind_id: KindId(2),
                client_key: key.into(),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields: field_patch("label", "label", key),
            },
        ))),
        request.allocation_policy(),
    )
    .unwrap();
    let result = txn.commit(request, runtime).unwrap();
    let relation = super::current_publications::changed_relations(&result)[0];
    release_test_commit_snapshot(runtime, &result);
    relation
}
