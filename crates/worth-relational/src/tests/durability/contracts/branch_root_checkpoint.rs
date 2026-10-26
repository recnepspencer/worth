use super::*;

#[test]
fn checkpoint_recovery_rejects_missing_exact_branch_root_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "root-artifact-required");
    crate::tests::support::checkpoint_durably(execution, &runtime)
        .expect("checkpoint captures the live main root");
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    plan.checkpoint
        .as_mut()
        .expect("checkpoint is selected")
        .branch_roots
        .clear();

    let mut recovered = durable_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("global partition images cannot substitute for a branch root");
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(
        error.detail.contains("missing branch-root image"),
        "{error:?}"
    );
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn checkpoint_recovery_rejects_root_artifact_without_commit_envelope() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "root-envelope-binding");
    crate::tests::support::checkpoint_durably(execution, &runtime)
        .expect("checkpoint captures the live main root");
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let checkpoint = plan.checkpoint.as_mut().expect("checkpoint is selected");
    checkpoint.branch_roots[0].commit_id = crate::history::data::CommitId(u64::MAX);

    let mut recovered = durable_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect_err("a root image cannot select an unrelated or missing envelope");
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("names missing commit envelope"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn checkpoint_recovery_rejects_duplicate_commit_envelopes_before_root_readmission() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "duplicate-envelope-root");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let checkpoint = plan.checkpoint.as_mut().expect("selected checkpoint");
    checkpoint.envelopes.push(checkpoint.envelopes[0].clone());

    let mut recovered = durable_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(
        error
            .detail
            .contains("reused by multiple canonical artifacts")
            || error
                .detail
                .contains("duplicate checkpoint commit envelope"),
        "{error:?}"
    );
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

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

#[test]
fn checkpoint_recovery_rejects_duplicate_global_partition_images() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "duplicate-global-partition");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let checkpoint = plan.checkpoint.as_mut().expect("selected checkpoint");
    checkpoint
        .partition_images
        .push(checkpoint.partition_images[0].clone());

    let mut recovered = durable_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("duplicate partition image"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
}

#[test]
fn checkpoint_recovery_rejects_duplicate_branch_root_partition_images() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "duplicate-root-partition");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let mut plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let root = &mut plan.checkpoint.as_mut().unwrap().branch_roots[0];
    root.partition_images.push(root.partition_images[0].clone());

    let mut recovered = durable_runtime_with_test_schema();
    let error = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err();
    let error = crate::tests::support::expect_durability_failure(error);

    assert_eq!(error.class, RecoveryFailureClass::CorruptCheckpoint);
    assert!(error.detail.contains("duplicate partition image"));
    assert_eq!(recovered.history().immutable_commit_count(), 0);
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

#[path = "branch_root_checkpoint/foreign_targets.rs"]
mod foreign_targets;
