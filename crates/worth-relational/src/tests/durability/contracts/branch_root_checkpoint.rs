use super::*;

#[test]
fn checkpoint_recovery_readmits_shared_root_once_for_sibling_heads() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "shared-root-seed");
    let sibling = create_branch_from_main(&runtime, "sibling");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    assert_eq!(plan.checkpoint.as_ref().unwrap().branch_roots.len(), 1);

    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    assert_eq!(
        current_branch_entity_count(execution, &recovered, &BranchId("main".to_owned())),
        1
    );
    assert_eq!(
        current_branch_entity_count(execution, &recovered, &sibling),
        1
    );
    let main = recovered
        .branch_reference_state(&BranchId("main".to_owned()))
        .unwrap();
    let sibling = recovered.branch_reference_state(&sibling).unwrap();
    assert_eq!(main.observation().target(), sibling.observation().target());
    let main_root = recovered
        .history
        .branch_cell(&BranchId("main".to_owned()))
        .unwrap()
        .root()
        .unwrap();
    let sibling_root = recovered
        .history
        .branch_cell(sibling.branch_id())
        .unwrap()
        .root()
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&main_root, &sibling_root));
}

#[test]
fn tail_recovery_resolves_the_checkpoint_target_root_not_the_fork_source_head() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "shared-seed");
    let storm = create_branch_from_main(&runtime, "storm");
    crate::tests::support::checkpoint_durably(execution, &runtime)
        .expect("checkpoint retains the shared pre-divergence root");

    create_entity_outcome(execution, &runtime, "main-tail-only");
    create_entity_outcome_on_branch(execution, &runtime, "storm-tail-only", storm.clone());
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();

    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect("divergent tail commits recover from their exact pre-commit roots");

    let main_count =
        current_branch_entity_count(execution, &recovered, &BranchId("main".to_owned()));
    let storm_count = current_branch_entity_count(execution, &recovered, &storm);
    assert_eq!(main_count, 2, "main contains seed plus its own tail write");
    assert_eq!(
        storm_count, 2,
        "storm excludes the source's later tail write"
    );
}

fn current_branch_entity_count(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch_id: &BranchId,
) -> usize {
    let identity = runtime
        .branch_identity(branch_id)
        .expect("recovered branch identity exists");
    let snapshot = crate::tests::support::snapshot_for_owner_identity(runtime, &identity);
    runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .expect("read request remains live")
        .expect("recovered branch snapshot is readable")
        .entities()
        .len()
}
