//! Actual entity, relation and generated-group authoring for current-read courts.
use super::current_field_meanings::*;
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
) -> EntityId {
    let committed =
        worth_testkit_relational::durable_publication::create_entity_outcome(request, runtime, key);
    let entity = changed_entities(&committed)[0];
    release_test_commit_snapshot(runtime, &committed);
    entity
}
pub(super) fn create_relation_outcome(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    key: &str,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("relation").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: key.into(),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                    fields: field_patch("label", "label", key),
                }),
            )),
            request.allocation_policy(),
        )
        .unwrap();
    transaction.commit(request, runtime).unwrap()
}
pub(super) fn changed_relations(committed: &CommitResult) -> Vec<RelationId> {
    committed
        .changed_records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Entity(_) => None,
            RecordRef::Relation(relation) => Some(*relation),
        })
        .collect()
}
pub(super) fn delete_entity(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: EntityId,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("delete").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent { entity_id }),
            )),
            request.allocation_policy(),
        )
        .unwrap();
    transaction.commit(request, runtime).unwrap()
}

pub(super) fn create_group_with_external_endpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    external: EntityId,
) -> CommitResult {
    let target = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: worth_relational::facade::symbols::ClientKey::raw("internal-endpoint"),
    };
    let batch = WorkerIntentBatch::new("open-generated-group")
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields: name_field_patch("internal-endpoint"),
        })))
        .push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: worth_relational::facade::symbols::ClientKey::raw("external-edge"),
                source: EntityReference::Existing(external),
                target: EntityReference::Created(target),
                fields: relation_label_field_patch("external-edge"),
            },
        )));
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the open generated group fits transaction budgets");
    transaction
        .commit(execution, runtime)
        .expect("the open generated group is otherwise a valid all-create publication")
}
