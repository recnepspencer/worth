use crate::facade::history::BranchId;
use crate::facade::merge::{MergeExecutionError, MergeExecutionRequest, MergeIntent};
use crate::tests::support::{
    create_branch_from_main, create_entity, create_entity_outcome_on_branch,
    durable_runtime_with_test_schema,
};

use super::schema_fixtures::drifted_schema_registry;

#[test]
fn authoritative_merge_execution_certification_rejects_stale_prepared_merge_after_target_advances()
{
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

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge execution");
    create_entity(execution, &runtime, "main-advance");

    match runtime.execute_prepared_merge(execution, prepared) {
        Err(MergeExecutionError::StaleBranchHead { branch, .. }) => {
            assert_eq!(branch, BranchId("main".to_string()));
        }
        other => panic!("expected target stale-head rejection, got {other:?}"),
    }
}

#[test]
fn authoritative_merge_execution_certification_rejects_stale_prepared_merge_after_source_advances()
{
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

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge execution");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-advance",
        BranchId("feature".to_string()),
    );

    match runtime.execute_prepared_merge(execution, prepared) {
        Err(MergeExecutionError::StaleBranchHead { branch, .. }) => {
            assert_eq!(branch, BranchId("feature".to_string()));
        }
        other => panic!("expected source stale-head rejection, got {other:?}"),
    }
}

#[test]
fn authoritative_merge_execution_certification_rejects_metadata_reference_movement() {
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

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge execution");
    let target_branch = BranchId("main".to_string());
    let target_head = runtime
        .history()
        .branch_head(&target_branch)
        .expect("main has a committed head");
    let target_generation = runtime
        .history
        .branch_cell(&target_branch)
        .expect("main has a reference cell")
        .observation()
        .generation()
        .get();
    runtime
        .history
        .branch_cell_mut(&target_branch)
        .expect("main has a mutable reference cell")
        .advance_metadata()
        .expect("metadata movement should advance only the reference generation");
    assert_eq!(
        runtime
            .history()
            .branch_head(&target_branch)
            .expect("metadata movement does not change the immutable head")
            .commit_id,
        target_head.commit_id
    );
    assert_eq!(
        runtime
            .history
            .branch_cell(&target_branch)
            .expect("main reference remains live")
            .observation()
            .generation()
            .get(),
        target_generation + 1
    );

    match runtime
        .merge()
        .verify_prepared_merge_execution(execution, &prepared)
    {
        Err(MergeExecutionError::StaleBranchReference { branch, .. }) => {
            assert_eq!(branch, target_branch);
        }
        other => panic!("expected metadata-only reference movement rejection, got {other:?}"),
    }
}

#[test]
fn authoritative_merge_execution_certification_rejects_schema_semantic_drift_after_planning() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "root");
    create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-only",
        BranchId("feature".to_string()),
    );

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared merge execution");
    runtime.set_schema_registry_for_test(drifted_schema_registry());

    match runtime.execute_prepared_merge(execution, prepared) {
        Err(MergeExecutionError::SchemaSemanticDrift { .. }) => {}
        other => panic!("expected schema semantic drift rejection, got {other:?}"),
    }
}
