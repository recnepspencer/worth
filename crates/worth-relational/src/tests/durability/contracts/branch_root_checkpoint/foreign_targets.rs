use super::*;

#[test]
fn checkpoint_recovery_rejects_foreign_branch_target_substitution() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "lineage-seed");
    let feature = create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(execution, &runtime, "feature-only", feature.clone());
    create_entity_outcome(execution, &runtime, "main-only");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let checkpoint = plan.checkpoint.as_mut().unwrap();
    let foreign_target = checkpoint
        .branch_cells
        .iter()
        .find(|cell| cell.branch_id == feature)
        .unwrap()
        .observation
        .target()
        .clone();
    let main = checkpoint
        .branch_cells
        .iter_mut()
        .find(|cell| cell.branch_id.0 == "main")
        .unwrap();
    replace_checkpoint_target(main, foreign_target);

    let mut recovered = durable_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("foreign branch stream"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn tail_recovery_rejects_foreign_branch_target_substitution() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "tail-lineage-seed");
    let feature = create_branch_from_main(&runtime, "feature");
    create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-before-checkpoint",
        feature.clone(),
    );
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let tail_commit = create_entity_outcome(execution, &runtime, "main-tail");
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let foreign_target = plan
        .checkpoint
        .as_ref()
        .unwrap()
        .branch_cells
        .iter()
        .find(|cell| cell.branch_id == feature)
        .unwrap()
        .observation
        .target()
        .clone();
    let tail_cell = plan
        .tail_log
        .iter_mut()
        .map(|commit| commit.envelope_mut_for_test())
        .find(|envelope| envelope.commit.commit_id == tail_commit.commit.commit_id)
        .unwrap()
        .branch_cell_checkpoint
        .as_mut()
        .unwrap();
    replace_checkpoint_target(tail_cell, foreign_target);

    let mut recovered = durable_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("foreign branch stream"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

fn replace_checkpoint_target(
    checkpoint: &mut crate::branch::RelationalBranchCellCheckpoint,
    target: worth_foundational::FoundationalBranchTarget<crate::branch::RelationalBranchTarget>,
) {
    checkpoint.observation = crate::branch::relational_branch_observation(
        checkpoint.runtime_instance_id,
        &checkpoint.branch_id.0,
        target,
        checkpoint.observation.generation(),
    )
    .unwrap();
}
