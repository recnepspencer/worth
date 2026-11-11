//! Shipping owner-bound planning keeps a cold nonhead source overlap.
use super::{checkpoint_receiver::reopen, merge_publications};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::BranchId,
    identity::{EntityId, PartitionId},
    merge::{
        MergeExecutionPreparationError, MergeExecutionRequest, MergeIntent, MergePlanningRequest,
    },
    runtime::RelationalRuntime,
    transactions::{
        CommitResult, EntityMutationIntent, MutationIntent, RecordRef, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::publications::release_test_commit_snapshot,
    cold_recovery::capture_source,
    durable_medium::TestDurableMedium,
    durable_publication::{durable_runtime, field_patch},
};

fn update(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: EntityId,
    value: &str,
    branch: &BranchId,
) -> CommitResult {
    let mut txn = merge_publications::transaction(runtime, branch);
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

#[test]
fn cold_intermediate_source_commit_remains_in_shipping_merge_overlap() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_runtime(&medium);
    let target = BranchId("main".into());
    let a = merge_publications::create(
        request,
        &producer,
        "merge-a-base",
        PartitionId::main(),
        &target,
    );
    let b = merge_publications::create(
        request,
        &producer,
        "merge-b-base",
        PartitionId::main(),
        &target,
    );
    let source = merge_publications::branch(request, &producer, "intermediate-source");
    let intermediate = update(request, &producer, a, "merge-a-source", &source);
    let commit = intermediate.commit.commit_id;
    let position = intermediate.patch_position();
    release_test_commit_snapshot(&producer, &intermediate);
    let head = update(request, &producer, b, "merge-b-source", &source);
    assert_ne!(head.commit.commit_id, commit);
    release_test_commit_snapshot(&producer, &head);
    let target_update = update(request, &producer, a, "merge-a-target", &target);
    let target_commit = target_update.commit.commit_id;
    release_test_commit_snapshot(&producer, &target_update);
    let original = capture_source(request, producer, medium, a, false);
    let recovered = reopen(request, &original);
    let runtime = &recovered.runtime;
    let frontier = runtime.history().latest_patch_stream_position();
    let offered = recovered.medium.offered().len();
    assert!(
        !recovered
            .source
            .commit_reads
            .lock()
            .unwrap()
            .contains(&position),
        "the intermediate original remains cold after root admission: {:?}",
        recovered.source.commit_reads.lock().unwrap(),
    );
    let bound = runtime
        .bind_merge_planning_request(MergePlanningRequest::new(
            target.clone(),
            source.clone(),
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap();
    let observation = runtime
        .merge()
        .inspect_planning_scope_with_contributions(request, bound)
        .unwrap();
    let record = RecordRef::Entity(a);
    assert!(observation
        .source_contributing_commits(record.clone())
        .any(|id| id == commit));
    assert!(observation
        .target_contributing_commits(record.clone())
        .any(|id| id == target_commit));
    assert!(observation
        .artifact()
        .conflict_classification
        .classifications
        .iter()
        .any(|row| row.record == record));
    assert!(
        recovered
            .source
            .commit_reads
            .lock()
            .unwrap()
            .contains(&position),
        "the planner demands the actual cold intermediate original"
    );
    let bound = runtime
        .bind_merge_execution_request(MergeExecutionRequest::new(
            target,
            source,
            MergeIntent::ReconcileIntoTarget,
        ))
        .unwrap();
    match runtime.prepare_merge_execution(request, bound) {
        Err(MergeExecutionPreparationError::NotExecutionReady(report)) => {
            assert!(report.denied_records.iter().any(|row| row.record == record));
        }
        other => panic!("the actual overlap needs explicit resolution: {other:?}"),
    }
    assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
    assert_eq!(recovered.medium.offered().len(), offered);
}
