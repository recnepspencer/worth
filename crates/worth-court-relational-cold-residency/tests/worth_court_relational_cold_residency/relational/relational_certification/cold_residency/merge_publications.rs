//! Actual registered branch authoring and owner-bound merge selection.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::BranchId,
    identity::*,
    merge::{MergeExecutionRequest, MergeIntent, OwnerBoundMergeExecutionRequest},
    mvcc::{BranchBoundRelationalTransaction, RelationalTransactionIntent},
    runtime::RelationalRuntime,
    transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::publications::{changed_entities, release_test_commit_snapshot},
    durable_publication::field_patch,
};
pub(super) fn branch(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    name: &str,
) -> BranchId {
    let id = BranchId(name.into());
    let (_, basis) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    runtime.fork_branch(request, id.clone(), basis).unwrap();
    id
}
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
pub(super) fn create(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    partition_id: PartitionId,
    branch: &BranchId,
) -> EntityId {
    let mut txn = transaction(runtime, branch);
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
pub(super) fn request(runtime: &RelationalRuntime) -> OwnerBoundMergeExecutionRequest {
    runtime
        .bind_merge_execution_request(MergeExecutionRequest::new(
            BranchId("main".into()),
            BranchId("cold-merge-feature".into()),
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap()
}
