//! Exact original branch publication recipes for checkpoint root journeys.
use super::branch_history_publications::transaction;
use super::*;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_relational::facade::{history::BranchId, runtime::RelationalRuntime, transactions::*};
use worth_testkit_relational::aspect_storage::publications::changed_entities;

pub(super) fn create_entity(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
) -> worth_relational::facade::identity::EntityId {
    let result = create_entity_outcome(execution, runtime, key);
    let id = changed_entities(&result)[0];
    release_test_commit_snapshot(runtime, &result);
    id
}
pub(super) fn create_on_branch(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    branch: BranchId,
) -> CommitResult {
    let mut txn = transaction(runtime, &branch);
    txn.push_batch(batch_create(key), Allocation::SystemAllocation)
        .unwrap();
    txn.commit(execution, runtime).unwrap()
}
pub(super) fn create_in_partition(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    partition_id: PartitionId,
) {
    let mut txn = transaction(runtime, &BranchId("main".into()));
    txn.push_batch(
        WorkerIntentBatch::new(format!("batch-{key}")).push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id,
                kind_id: KindId(1),
                client_key: key.into(),
                fields: field_patch("name", "name", key),
            }),
        )),
        Allocation::SystemAllocation,
    )
    .unwrap();
    let result = txn.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &result);
}
pub(super) fn update_entity(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: worth_relational::facade::identity::EntityId,
    key: &str,
) {
    let mut txn = transaction(runtime, &BranchId("main".into()));
    txn.push_batch(
        WorkerIntentBatch::new("update").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id,
                fields: field_patch("name", "name", key),
            }),
        )),
        Allocation::SystemAllocation,
    )
    .unwrap();
    let result = txn.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &result);
}
