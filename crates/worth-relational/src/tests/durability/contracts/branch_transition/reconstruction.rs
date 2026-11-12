//! Replay and recovery reconstruct only the history absent from their basis.
use super::*;
use crate::facade::replay::{RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode};

fn replay(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    commit: crate::history::data::CommitId,
    branch: &str,
) {
    let replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: commit,
                branch_id: BranchId(branch.into()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    assert!(replay.mismatches.is_empty(), "{replay:?}");
}

#[test]
fn memory_checkpoint_keeps_replay_reconstructible() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = runtime_with_test_schema_profile(
        crate::config::data::RelationalRuntimeProfile::GeometryKernel,
    );
    assert_eq!(
        runtime.config().durability.policy.log.retention_mode,
        crate::config::data::DurableLogRetentionMode::CompactAfterCheckpoint
    );
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "A");
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let target = create_entity_outcome(execution, &runtime, "after-checkpoint");
    replay(execution, &runtime, target.commit.commit_id, "main");
    // This earlier chain cannot select the later checkpoint: memory has no
    // journal from which to restore its covered fork.
    let on_a = create_entity_outcome_on_branch(execution, &runtime, "on-A", BranchId("A".into()));
    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    replay(execution, &runtime, target.commit.commit_id, "main");
    replay(execution, &runtime, on_a.commit.commit_id, "A");
}
