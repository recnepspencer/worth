use super::native_strategy_fixtures::*;

#[test]
fn replay_commit_reports_strategy_executor_unavailable_when_recovered_runtime_lacks_executor() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let root_path = unique_test_store_path("worth-relational-strategy-replay-missing-executor");
    let mut runtime = persisted_intent_runtime(root_path.clone(), true);
    let entity = crate::tests::support::create_entity(&runtime, "before");
    let commit = execute_persisted_intent_strategy_commit(execution, &mut runtime, entity);
    let branch_head_before = runtime.history().branch_head(&BranchId("main".to_string()));
    let mut recovery_plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::AuditRecoveryVerification,
    );
    recovery_plan.commit_strategy_executors = Default::default();

    let mut recovered = persisted_intent_runtime(root_path, false);
    recovered
        .durability_recovery()
        .recover(execution, recovery_plan)
        .expect("recovery without strategy executor");
    let branch_head_after_recovery = recovered
        .history()
        .branch_head(&BranchId("main".to_string()));

    let replay = recovered.replay_authority().replay_commit(
        RelationalReplayRequest {
            commit_id: commit.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
        },
        execution,
    );

    assert_eq!(replay.failure, Some(ReplayFailureClass::ObservableMismatch));
    assert!(replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Strategy));
    assert!(replay.mismatches.iter().any(|mismatch| {
        mismatch.class == ReplayMismatchClass::StrategyExecutorUnavailable
            && mismatch.surface == ReplayObservableSurface::Strategy
    }));
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_string())),
        branch_head_after_recovery
    );
    assert_eq!(branch_head_after_recovery, branch_head_before);
}

#[test]
fn replay_commit_reports_strategy_execution_failure_when_recovered_executor_rejects() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let root_path = unique_test_store_path("worth-relational-strategy-replay-failing-executor");
    let mut runtime = persisted_intent_runtime(root_path.clone(), true);
    let entity = crate::tests::support::create_entity(&runtime, "before");
    let commit = execute_persisted_intent_strategy_commit(execution, &mut runtime, entity);
    let mut recovery_plan = runtime.durability().recovery_plan(
        crate::durability::data::RecoveryVerificationMode::AuditRecoveryVerification,
    );

    let mut recovered = persisted_intent_runtime_with_failing_executor(root_path);
    recovery_plan.commit_strategy_executors = recovered.commit_strategy_executor_registry().clone();
    recovered
        .durability_recovery()
        .recover(execution, recovery_plan)
        .expect("recovery with hostile failing executor");
    let branch_head_before_replay = recovered
        .history()
        .branch_head(&BranchId("main".to_string()));

    let replay = recovered.replay_authority().replay_commit(
        RelationalReplayRequest {
            commit_id: commit.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
        },
        execution,
    );

    assert_eq!(replay.failure, Some(ReplayFailureClass::ObservableMismatch));
    assert!(replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Strategy));
    assert!(replay.mismatches.iter().any(|mismatch| {
        mismatch.class == ReplayMismatchClass::StrategyExecutionFailure
            && mismatch.surface == ReplayObservableSurface::Strategy
    }));
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_string())),
        branch_head_before_replay
    );
}
