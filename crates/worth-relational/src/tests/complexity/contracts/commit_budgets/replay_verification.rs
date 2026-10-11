use super::*;

#[test]
fn complexity_budget_replay_verification_tracks_digest_and_deep_layers_separately() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "replayable");

    runtime.performance_access().reset_counters();
    let normal = runtime.replay_authority().replay_commit(
        crate::replay::data::RelationalReplayRequest {
            commit_id: created.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: crate::replay::data::ReplayExecutionMode::SerialDeterministic,
            verification_mode:
                crate::replay::data::ReplayVerificationMode::NormalRecoveryVerification,
        },
        execution,
    );
    assert!(runtime.replay().compare_outcome(&normal));
    let normal_counters = runtime.performance_access().counters();
    assert!(normal_counters.replay_digest_parity_checks > 0);
    assert_eq!(normal_counters.replay_deep_artifact_parity_checks, 0);

    runtime
        .history_authority()
        .tamper_commit_patch_for_test(created.commit.commit_id, |patch| {
            patch.authoritative_record_patches.clear();
        });

    runtime.performance_access().reset_counters();
    let audited = runtime.replay_authority().replay_commit(
        crate::replay::data::RelationalReplayRequest {
            commit_id: created.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: crate::replay::data::ReplayExecutionMode::SerialDeterministic,
            verification_mode:
                crate::replay::data::ReplayVerificationMode::AuditRecoveryVerification,
        },
        execution,
    );
    assert_eq!(
        audited.failure,
        Some(crate::replay::data::ReplayFailureClass::ObservableMismatch)
    );
    let audit_counters = runtime.performance_access().counters();
    assert!(audit_counters.replay_digest_parity_checks > 0);
    assert!(audit_counters.replay_deep_artifact_parity_checks > 0);
}
