use crate::facade::history::BranchId;
use crate::facade::merge::{
    MergeExecutableClass, MergeExecutionCompilationError, MergeExecutionRequest, MergeIntent,
    MergeResolutionClass, TopologyExecutionClass,
};
use crate::facade::transactions::RecordRef;
use crate::tests::support::{
    changed_entities, create_branch_from_main, create_entity, create_entity_outcome_on_branch,
    durable_runtime_with_test_schema,
};

#[test]
fn admitted_source_addition_carries_executable_class() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
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
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            }
            .into(),
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

#[test]
fn compile_rejects_corrupted_non_executable_resolution_class() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let mut prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge");

    let execution_ready = prepared.execution_ready_plan_mut_for_test();
    let lowered = std::sync::Arc::make_mut(&mut execution_ready.lowered_records);
    lowered[0].resolution_class =
        MergeResolutionClass::Topology(TopologyExecutionClass::TopologyRegionConflict);
    lowered[0].executable_class = None;

    match runtime
        .merge()
        .compile_execution_ready_merge_plan_for_test(execution_ready)
    {
        Err(MergeExecutionCompilationError::MissingExecutableClass { .. }) => {}
        other => panic!("expected missing executable class rejection, got {other:?}"),
    }
}
