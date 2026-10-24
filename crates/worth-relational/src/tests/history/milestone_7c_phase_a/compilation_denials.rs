use std::sync::Arc;

use crate::facade::history::BranchId;
use crate::facade::merge::{MergeExecutionError, MergeExecutionRequest, MergeIntent};
use crate::merge::data::MergeExecutionCompilationError;
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch,
    durable_runtime_with_test_schema,
};

#[test]
fn compile_execution_ready_merge_plan_rejects_missing_source_record() {
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
    prepared.execution_ready_plan_mut_for_test().source_records = Arc::from([]);

    match runtime
        .merge()
        .compile_execution_ready_merge_plan_for_test(prepared.execution_ready_plan())
    {
        Err(MergeExecutionCompilationError::MissingSourceRecord { .. }) => {}
        other => panic!("expected missing source record compilation failure, got {other:?}"),
    }
}

#[test]
fn verify_prepared_merge_execution_rejects_corrupted_compiled_plan() {
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
    prepared.bound_executable_plan_mut_for_test().record_plans = Arc::from([]);

    match runtime
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
    {
        Err(MergeExecutionError::Compilation(
            MergeExecutionCompilationError::PreparedAuthorityBindingMismatch { .. },
        )) => {}
        other => panic!("expected compilation rejection during verify, got {other:?}"),
    }
}
