use super::branch_history_publications::{fork, transaction};
use super::merge_execution_authoring::single_string_aspect_field_patch;
use worth_relational::facade::{
    history::BranchId,
    identity::{KindId, PartitionId},
    merge::{MergeExecutionRequest, MergeIntent},
    symbols::ClientKey,
    transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};
use worth_testkit_relational::{
    aspect_storage::{
        definition::{aspect_key, field_key},
        publications::changed_entities,
    },
    durable_medium::TestDurableMedium,
    durable_publication::{create_entity, durable_runtime},
};

#[test]
fn source_only_merge_emits_one_new_entity_with_an_independent_identity() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "main-anchor");
    fork(execution, &runtime, "feature");
    let mut txn = transaction(&runtime, &BranchId("feature".into()));
    txn.push_batch(
        WorkerIntentBatch::new("create-feature-only").push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("feature-only"),
                fields: single_string_aspect_field_patch(
                    aspect_key("name"),
                    field_key("name"),
                    "feature-only",
                ),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let feature_only = changed_entities(
        &txn.commit(execution, &runtime)
            .expect("feature-only create"),
    )[0];
    let prepared = runtime
        .prepare_merge_execution(
            execution,
            runtime
                .bind_merge_execution_request(MergeExecutionRequest::new(
                    BranchId("main".into()),
                    BranchId("feature".into()),
                    MergeIntent::ReconcileIntoTarget,
                ))
                .unwrap(),
        )
        .expect("prepared merge execution");
    let outcome = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed merge");
    assert_eq!(outcome.structural_summary.adopted_source_record_count, 1);
    assert_eq!(outcome.structural_summary.emitted_entity_create_count, 1);
    assert_eq!(changed_entities(&outcome.commit).len(), 1);
    assert_ne!(changed_entities(&outcome.commit)[0], feature_only);
}
