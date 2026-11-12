//! An admitted source-only addition carries its executable class in the lowered plan.
use super::branch_history_publications::{create_entity, create_entity_outcome_on_branch, fork};
use worth_relational::facade::{
    history::BranchId,
    merge::{MergeExecutableClass, MergeExecutionRequest, MergeIntent, MergeResolutionClass},
    transactions::RecordRef,
};
use worth_testkit_relational::{
    aspect_storage::publications::changed_entities, durable_medium::TestDurableMedium,
    durable_publication::durable_runtime,
};

#[test]
fn admitted_source_addition_carries_executable_class() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "root");
    fork(execution, &runtime, "feature");
    let feature_only = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );
    let entity = changed_entities(&feature_only)[0];

    let artifact = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            runtime
                .bind_merge_planning_request(
                    MergeExecutionRequest::new(
                        BranchId("main".to_string()),
                        BranchId("feature".to_string()),
                        MergeIntent::ReconcileIntoTarget,
                    )
                    .into(),
                )
                .unwrap(),
        )
        .expect("planning artifact");

    let lowered = artifact
        .lowered_plan
        .records
        .iter()
        .find(|record| record.record == RecordRef::Entity(entity))
        .expect("lowered record");

    assert_eq!(
        lowered.resolution_class,
        MergeResolutionClass::SourceOnlyAddition
    );
    assert_eq!(
        lowered.executable_class,
        Some(MergeExecutableClass::AdoptSourceRecord)
    );
}
