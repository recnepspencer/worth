use super::*;

pub(super) fn certify(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    recovery_plan: &crate::durability::data::RecoveryPlan,
    recovered_root: &std::path::PathBuf,
    commit_id: crate::facade::history::CommitId,
) -> (bool, bool) {
    let mut missing_executor_plan = recovery_plan.clone();
    missing_executor_plan.commit_strategy_executors = Default::default();
    let mut missing_executor_runtime =
        persisted_strategy_runtime_without_executors(recovered_root.clone());
    missing_executor_runtime
        .durability_recovery()
        .recover(execution, missing_executor_plan)
        .expect("recover without executors");
    let missing_executor_replay = missing_executor_runtime.replay_authority().replay_commit(
        RelationalReplayRequest {
            commit_id: commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
        },
        execution,
    );
    assert!(missing_executor_replay.mismatches.iter().any(|mismatch| {
        mismatch.class == ReplayMismatchClass::StrategyExecutorUnavailable
            && mismatch.surface == ReplayObservableSurface::Strategy
    }));
    let missing_executor_mismatch_present =
        missing_executor_replay.mismatches.iter().any(|mismatch| {
            mismatch.class == ReplayMismatchClass::StrategyExecutorUnavailable
                && mismatch.surface == ReplayObservableSurface::Strategy
        });

    let mut failing_executor_runtime =
        persisted_strategy_runtime_with_failing_intent_executor(recovered_root.clone());
    let mut failing_executor_plan = recovery_plan.clone();
    failing_executor_plan.commit_strategy_executors = failing_executor_runtime
        .commit_strategy_executor_registry()
        .clone();
    failing_executor_runtime
        .durability_recovery()
        .recover(execution, failing_executor_plan)
        .expect("recover with failing intent executor");
    let failing_executor_replay = failing_executor_runtime.replay_authority().replay_commit(
        RelationalReplayRequest {
            commit_id: commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
        },
        execution,
    );
    assert!(failing_executor_replay.mismatches.iter().any(|mismatch| {
        mismatch.class == ReplayMismatchClass::StrategyExecutionFailure
            && mismatch.surface == ReplayObservableSurface::Strategy
    }));
    let failing_executor_mismatch_present =
        failing_executor_replay.mismatches.iter().any(|mismatch| {
            mismatch.class == ReplayMismatchClass::StrategyExecutionFailure
                && mismatch.surface == ReplayObservableSurface::Strategy
        });
    (
        missing_executor_mismatch_present,
        failing_executor_mismatch_present,
    )
}
