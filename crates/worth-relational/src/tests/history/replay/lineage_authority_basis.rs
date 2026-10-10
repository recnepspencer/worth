use super::*;

fn replay_request(commit_id: crate::history::data::CommitId) -> RelationalReplayRequest {
    RelationalReplayRequest {
        commit_id,
        branch_id: BranchId("main".to_owned()),
        execution_mode: ReplayExecutionMode::SerialDeterministic,
        verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
    }
}

#[test]
fn replay_reports_owner_create_event_drift_at_digest_layer() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "event-drift");
    assert!(runtime.history_authority().tamper_commit_envelope_for_test(
        created.commit.commit_id,
        |envelope| {
            envelope
                .published_lineage_mut_for_test()
                .lineage_events_mut()[0]
                .kind = crate::facade::lineage::LineageEventKind::Retire;
        }
    ));

    let replay = runtime
        .replay_authority()
        .replay_commit(replay_request(created.commit.commit_id), execution);

    assert_eq!(replay.failure, Some(ReplayFailureClass::ObservableMismatch));
    assert!(replay.mismatches.iter().any(|mismatch| {
        mismatch.class == ReplayMismatchClass::LineageDrift
            && mismatch.surface == ReplayObservableSurface::Lineage
            && mismatch.verification_layer
                == crate::facade::replay::ReplayVerificationLayer::DigestParity
    }));
}

#[test]
fn replay_reports_owner_create_decision_drift_at_digest_layer() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "decision-drift");
    assert!(runtime.history_authority().tamper_commit_envelope_for_test(
        created.commit.commit_id,
        |envelope| {
            envelope
                .published_lineage_mut_for_test()
                .lineage_decision_log_mut()[0]
                .kind = crate::facade::lineage::LineageDecisionKind::RetireAccepted;
        }
    ));

    let replay = runtime
        .replay_authority()
        .replay_commit(replay_request(created.commit.commit_id), execution);

    assert_eq!(replay.failure, Some(ReplayFailureClass::ObservableMismatch));
    assert!(replay.mismatches.iter().any(|mismatch| {
        mismatch.class == ReplayMismatchClass::LineageDrift
            && mismatch.surface == ReplayObservableSurface::Lineage
            && mismatch.verification_layer
                == crate::facade::replay::ReplayVerificationLayer::DigestParity
    }));
}

#[test]
fn replay_uses_retained_owner_envelope_only_in_normal_mode() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "retained-envelope");
    assert!(runtime
        .durability_authority()
        .remove_durable_envelope_for_test(created.commit.commit_id));

    let replay = runtime
        .replay_authority()
        .replay_commit(replay_request(created.commit.commit_id), execution);

    assert!(runtime.replay().compare_outcome(&replay));
    assert_eq!(
        replay
            .lineage_authority_basis
            .as_ref()
            .map(|basis| basis.kind()),
        Some(crate::facade::replay::ReplayAuthorityBasisKind::RetainedEnvelopeCanonical)
    );
}

#[test]
fn audit_replay_rejects_retained_owner_envelope_basis() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "audit-retained-envelope");
    assert!(runtime
        .durability_authority()
        .remove_durable_envelope_for_test(created.commit.commit_id));
    let mut request = replay_request(created.commit.commit_id);
    request.verification_mode = ReplayVerificationMode::AuditRecoveryVerification;

    let replay = runtime.replay_authority().replay_commit(request, execution);

    assert_eq!(
        replay.failure,
        Some(ReplayFailureClass::AuthoritativeBasisUnavailable)
    );
}

#[test]
fn audit_replay_uses_checkpoint_owner_basis_when_tail_is_absent() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "checkpoint-basis");
    runtime.durability_authority().checkpoint().unwrap();
    assert!(runtime
        .durability_authority()
        .remove_durable_envelope_for_test(created.commit.commit_id));
    let mut request = replay_request(created.commit.commit_id);
    request.verification_mode = ReplayVerificationMode::AuditRecoveryVerification;

    let replay = runtime.replay_authority().replay_commit(request, execution);

    assert!(runtime.replay().compare_outcome(&replay));
    assert_eq!(
        replay
            .lineage_authority_basis
            .as_ref()
            .map(|basis| basis.kind()),
        Some(crate::facade::replay::ReplayAuthorityBasisKind::DurableLogCanonical)
    );
}
