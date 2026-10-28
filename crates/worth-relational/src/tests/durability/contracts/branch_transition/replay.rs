use super::*;
use crate::facade::replay::{RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode};

#[test]
fn partial_replay_includes_an_intermediate_fork_without_its_own_commit() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity(execution, &runtime, "source");
    fork(execution, &runtime, "A");
    let (_, source) = runtime.observe_fork_source(&BranchId("A".into())).unwrap();
    runtime
        .fork_branch(execution, BranchId("B".into()), source)
        .unwrap();
    let target =
        create_entity_outcome_on_branch(execution, &runtime, "target", BranchId("B".into()));
    let verify = |runtime: &RelationalRuntime| {
        let replay = runtime
            .replay_authority()
            .replay_commit(
                RelationalReplayRequest {
                    commit_id: target.commit.commit_id,
                    branch_id: BranchId("B".into()),
                    execution_mode: ReplayExecutionMode::SerialDeterministic,
                    verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
                },
                execution,
            )
            .unwrap();
        assert!(replay.mismatches.is_empty(), "{replay:?}");
    };
    verify(&runtime);
    checkpoint_into_medium(execution, &runtime);
    assert!(artifacts(&runtime).is_empty());
    let recovered = reopen(execution, &runtime);
    verify(&recovered);
}
