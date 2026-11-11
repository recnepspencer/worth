//! Ordinary owner-admitted graph publications for branch history recovery.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::BranchId,
    identity::*,
    mvcc::{BranchBoundRelationalTransaction, RelationalTransactionIntent},
    runtime::RelationalRuntime,
    transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::publications::{changed_entities, release_test_commit_snapshot},
    durable_publication::field_patch,
};

pub(super) fn transaction(
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> BranchBoundRelationalTransaction {
    let identity = runtime.branch_identity(branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap()
}
pub(super) fn fork(request: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, name: &str) {
    let (_, basis) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    runtime
        .fork_branch(request, BranchId(name.into()), basis)
        .unwrap();
}
pub(super) fn create_entity_outcome_on_branch(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    branch: BranchId,
) -> CommitResult {
    create_with_fields(
        request,
        runtime,
        key,
        &branch,
        field_patch("name", "name", key),
    )
}
pub(super) fn create_with_fields(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    branch: &BranchId,
    fields: AspectFieldPatch,
) -> CommitResult {
    let mut txn = transaction(runtime, branch);
    txn.push_batch(
        WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: key.into(),
                fields,
            },
        ))),
        request.allocation_policy(),
    )
    .unwrap();
    txn.commit(request, runtime).unwrap()
}
pub(super) fn create_entity(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
) -> EntityId {
    let created = create_entity_outcome_on_branch(request, runtime, key, BranchId("main".into()));
    let id = changed_entities(&created)[0];
    release_test_commit_snapshot(runtime, &created);
    id
}
pub(super) fn update_entity_on_branch(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: EntityId,
    value: &str,
    branch: BranchId,
) -> CommitResult {
    let mut txn = transaction(runtime, &branch);
    txn.push_batch(
        WorkerIntentBatch::new(value).push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: field_patch("name", "name", value),
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    txn.commit(request, runtime).unwrap()
}
pub(super) fn update_entity(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: EntityId,
    value: &str,
) -> CommitResult {
    update_entity_on_branch(request, runtime, entity, value, BranchId("main".into()))
}
pub(super) fn create_relation_outcome(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    key: &str,
) -> CommitResult {
    relation_with_fields(
        request,
        runtime,
        source,
        target,
        key,
        field_patch("label", "label", key),
    )
}
pub(super) fn relation_with_fields(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    key: &str,
    fields: AspectFieldPatch,
) -> CommitResult {
    let mut txn = transaction(runtime, &BranchId("main".into()));
    txn.push_batch(
        WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: key.into(),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
                fields,
            },
        ))),
        request.allocation_policy(),
    )
    .unwrap();
    txn.commit(request, runtime).unwrap()
}
pub(super) fn changed_relations(result: &CommitResult) -> Vec<RelationId> {
    result
        .changed_records
        .iter()
        .filter_map(|r| match r {
            RecordRef::Entity(_) => None,
            RecordRef::Relation(id) => Some(*id),
        })
        .collect()
}
pub(super) fn delete_entity(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: EntityId,
) -> CommitResult {
    let mut txn = transaction(runtime, &BranchId("main".into()));
    txn.push_batch(
        WorkerIntentBatch::new("delete-entity").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    txn.commit(request, runtime).unwrap()
}
pub(super) fn delete_relation_on_branch(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    relation_id: RelationId,
    branch: BranchId,
) -> CommitResult {
    let mut txn = transaction(runtime, &branch);
    txn.push_batch(
        WorkerIntentBatch::new("delete-relation").push(MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent { relation_id }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    txn.commit(request, runtime).unwrap()
}

pub(super) fn delete_entity_on_branch(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: EntityId,
    branch: BranchId,
) -> CommitResult {
    let mut txn = transaction(runtime, &branch);
    txn.push_batch(
        WorkerIntentBatch::new("delete-entity").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    txn.commit(request, runtime).unwrap()
}
