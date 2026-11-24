use crate::facade::history::BranchId;
use crate::facade::merge::{
    MergeExecutionCompilationError, MergeExecutionRequest, MergeIntent, MergeResolutionClass,
    TopologyExecutionClass,
};
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch,
    durable_runtime_for_medium, OwnedTestDurableMedium,
};

#[test]
fn compile_rejects_corrupted_non_executable_resolution_class() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = OwnedTestDurableMedium::new();
    let runtime = durable_runtime_for_medium(&medium);
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
            MergeExecutionRequest::new(
                BranchId("main".to_string()),
                BranchId("feature".to_string()),
                MergeIntent::ReconcileIntoTarget,
            ),
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
