use super::*;

#[test]
fn replay_contract_success_reproduces_canonical_surfaces() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let outcome = create_entity_outcome(execution, &runtime, "replayable");
    let replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: outcome.commit.commit_id,
                branch_id: BranchId("main".to_string()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
            },
            execution,
        )
        .unwrap();

    assert!(runtime.replay().compare_outcome(&replay));
    assert_eq!(
        replay.reconstructed_commit_closure,
        vec![outcome.commit.commit_id]
    );
    assert!(runtime
        .publication()
        .diagnostics()
        .by_scope(DiagnosticsScope::Replay)
        .iter()
        .any(|artifact| artifact.kind == DiagnosticsArtifactKind::Comparison));
}

#[test]
fn replay_contract_failure_wrong_branch_is_explicit() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let outcome = create_entity_outcome(execution, &runtime, "replayable");
    let replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: outcome.commit.commit_id,
                branch_id: BranchId("wrong".to_string()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
            },
            execution,
        )
        .unwrap();

    assert_eq!(replay.failure, Some(ReplayFailureClass::BranchMismatch));
}

#[test]
fn replay_uses_canonical_root_parent_when_catalog_accelerator_is_missing() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let parent = create_entity_outcome(execution, &runtime, "parent");
    let child = create_entity_outcome(execution, &runtime, "child");

    assert!(runtime
        .history_authority()
        .remove_commit_envelope_for_test(parent.commit.commit_id));

    let replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: child.commit.commit_id,
                branch_id: BranchId("main".to_string()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
            },
            execution,
        )
        .unwrap();

    assert!(runtime.replay().compare_outcome(&replay));
    assert_eq!(
        replay.reconstructed_commit_closure,
        vec![parent.commit.commit_id, child.commit.commit_id]
    );
}

#[test]
fn replay_contract_success_preserves_merge_parent_order() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let main = create_entity_outcome(execution, &runtime, "main");
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let feature = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature",
        BranchId("feature".to_string()),
    );
    let merge = merge_commit_from_branches(
        execution,
        &runtime,
        BranchId("main".to_string()),
        vec![BranchId("feature".to_string())],
    );
    let replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: merge.commit.commit_id,
                branch_id: BranchId("main".to_string()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
            },
            execution,
        )
        .unwrap();

    assert!(runtime.replay().compare_outcome(&replay));
    assert_eq!(
        runtime
            .replay()
            .canonical_commit_envelope(execution, merge.commit.commit_id)
            .unwrap()
            .map(|envelope| envelope.as_ref().clone())
            .unwrap()
            .commit
            .parents,
        vec![main.commit.commit_id, feature.commit.commit_id]
    );
    assert_eq!(
        runtime
            .replay()
            .canonical_commit_envelope(execution, merge.commit.commit_id)
            .unwrap()
            .map(|envelope| envelope.as_ref().clone())
            .unwrap()
            .merge_base_commits,
        vec![main.commit.commit_id]
    );
    assert!(replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::History));
}

#[test]
fn replay_contract_reports_structured_patch_drift_when_canonical_envelope_is_tampered() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let outcome = create_entity_outcome(execution, &runtime, "replayable");
    assert!(runtime.history_authority().tamper_commit_patch_for_test(
        outcome.commit.commit_id,
        |patch| {
            patch.authoritative_record_patches[0].detail = PatchDetail::DenseBitset(vec![99]);
        }
    ));

    let replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: outcome.commit.commit_id,
                branch_id: BranchId("main".to_string()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
            },
            execution,
        )
        .unwrap();

    assert_eq!(replay.failure, Some(ReplayFailureClass::ObservableMismatch));
    assert_eq!(replay.mismatches.len(), 1);
    assert_eq!(replay.mismatches[0].class, ReplayMismatchClass::PatchDrift);
    assert_eq!(replay.mismatches[0].surface, ReplayObservableSurface::Patch);
    assert_eq!(
        replay.mismatches[0].verification_layer,
        crate::facade::replay::ReplayVerificationLayer::DigestParity
    );
    assert!(replay.mismatches[0].expected.is_some());
    assert!(replay.mismatches[0].observed.is_some());
}

#[test]
fn replay_contract_reports_diagnostics_drift_at_digest_layer_when_envelope_is_tampered() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let outcome = create_entity_outcome(execution, &runtime, "replayable");
    assert!(runtime.history_authority().tamper_commit_envelope_for_test(
        outcome.commit.commit_id,
        |envelope| {
            envelope.diagnostics_summary.entries.clear();
        }
    ));

    let replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: outcome.commit.commit_id,
                branch_id: BranchId("main".to_string()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
            },
            execution,
        )
        .unwrap();

    assert_eq!(replay.failure, Some(ReplayFailureClass::ObservableMismatch));
    assert!(
        replay.mismatches.iter().any(|mismatch| {
            mismatch.class == ReplayMismatchClass::DiagnosticsDrift
                && mismatch.surface == ReplayObservableSurface::Diagnostics
                && mismatch.verification_layer
                    == crate::facade::replay::ReplayVerificationLayer::DigestParity
        }),
        "{:?}",
        replay.mismatches
    );
}
