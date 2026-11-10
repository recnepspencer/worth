//! Replay and recovery reconstruct only the history absent from their basis.
use super::*;
use crate::facade::durability::RecoveryVerificationMode;
use crate::facade::replay::{RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode};
use std::sync::Arc;

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

#[test]
fn recovery_plan_after_native_capture_keeps_covered_forks() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "A");
    let target = create_entity_outcome_on_branch(execution, &runtime, "on-A", BranchId("A".into()));
    let native = runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let contents = crate::tests::support::attached_durable_medium(&runtime).unwrap();
    contents.file_checkpoint(native);
    let selected = contents
        .recovery_source()
        .unwrap()
        .checkpoint()
        .cloned()
        .expect("actual selected native capture");
    let covered = selected.covered();
    let later = create_entity_outcome(execution, &runtime, "main-after-selected-checkpoint");
    let source: Arc<dyn crate::durability::medium::RelationalDurableColdSource> = contents;
    let protection =
        crate::durability::medium::RelationalDurableReadProtection::capture(&source, execution)
            .unwrap();
    assert_eq!(protection.bounds().canonical.0, covered.0 + 1);
    let original =
        crate::durability::medium::RelationalDurableRecoverySource::from_protected_source(
            execution,
            Some(selected),
            source,
            &protection,
        )
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            original,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("A".into()))
            .unwrap()
            .commit_id,
        target.commit.commit_id
    );
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".into()))
            .unwrap()
            .commit_id,
        later.commit.commit_id
    );
}

#[test]
fn a_memory_fork_can_replay_its_compacted_durable_source() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "A");
    let target = create_entity_outcome_on_branch(execution, &runtime, "on-A", BranchId("A".into()));
    runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let speculative = runtime.fork().unwrap();
    assert!(
        speculative.durability.medium().is_none(),
        "a runtime fork cannot publish to its source's medium"
    );
    replay(execution, &speculative, target.commit.commit_id, "A");
}
