mod admission_boundary;

use super::*;

#[test]
fn authority_denial_certification_retains_typed_zero_residue_proof() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let WritebackHarnessExecution::AuthorityDenialCertification {
        failure_digest,
        authority_denial,
        zero_residue_report,
        counter_snapshot,
    } = certified_execution(
        WritebackHarnessTarget::AuthorityDenialCertification,
        resource_request,
    )
    else {
        panic!("authority-denial certification should produce authority-denial typed matrix");
    };

    assert!(!failure_digest.is_empty());
    assert_eq!(
        authority_denial.validation_failure_kind(),
        crate::facade::BridgeWritebackErrorKind::PreviewWritebackRejected
    );
    assert_eq!(
        authority_denial
            .authority_boundary()
            .unbound_authority_failure()
            .effect()
            .expect("unbound authority denial must retain typed lowered effect")
            .effect_intent_digest(),
        authority_denial
            .authority_boundary()
            .unbound_authority_failure()
            .effect_intent_digest()
            .expect("unbound authority projection must derive from retained effect")
    );
    assert!(authority_denial
        .authority_boundary()
        .unsafe_feedback_failure()
        .effect()
        .is_some());
    let unsafe_feedback_failure = authority_denial
        .authority_boundary()
        .unsafe_feedback_failure();
    let unsafe_feedback_context = unsafe_feedback_failure
        .incoming_feedback_context()
        .expect("unsafe feedback denial must retain incoming feedback context");
    assert_eq!(
        unsafe_feedback_failure.feedback_provenance_digest(),
        Some(unsafe_feedback_context.provenance_digest())
    );
    assert_eq!(
        unsafe_feedback_failure.incoming_feedback_causality_digest(),
        Some(unsafe_feedback_context.causality_digest())
    );
    assert_eq!(
        authority_denial.unsafe_feedback_partial().digest(),
        authority_denial.unsafe_feedback_partial().report().digest()
    );
    assert_eq!(
        authority_denial
            .unsafe_feedback_partial()
            .current_feedback_provenance_digest(),
        authority_denial
            .unsafe_feedback_partial()
            .report()
            .current_feedback_provenance()
            .digest()
    );
    assert_eq!(
        authority_denial
            .unsafe_feedback_partial()
            .incoming_feedback_causality_digest(),
        authority_denial
            .unsafe_feedback_partial()
            .report()
            .incoming_feedback_context()
            .map(crate::facade::BridgeWritebackFeedbackContext::causality_digest)
    );
    assert_eq!(
        authority_denial.unsafe_feedback_partial().disposition(),
        crate::facade::BridgeWritebackLoopDisposition::RejectAsUnsafeFeedback
    );
    assert_eq!(zero_residue_report.authoritative_commit_count(), 0);
    assert_eq!(zero_residue_report.authoritative_artifact_count(), 0);
    assert_eq!(counter_snapshot.writeback_failure_count, 3);
    assert_eq!(counter_snapshot.writeback_validation_rejection_count, 2);
}

#[test]
fn replay_mismatch_certification_retains_typed_effect_intent_proof() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let WritebackHarnessExecution::ReplayMismatchCertification {
        replay_validation_digest,
        replay_mismatch_matrix,
        counter_snapshot,
    } = certified_execution(
        WritebackHarnessTarget::ReplayMismatchCertification,
        resource_request,
    )
    else {
        panic!("replay mismatch should produce replay typed matrix");
    };

    assert!(!replay_validation_digest.is_empty());
    assert_eq!(
        replay_mismatch_matrix.failure_kind(),
        crate::facade::BridgeWritebackErrorKind::ReplayMismatch
    );
    assert!(replay_mismatch_matrix.semantic_mismatch_detected());
    assert_ne!(
        replay_mismatch_matrix.expected_effect_intent_digest(),
        replay_mismatch_matrix.replayed_effect_intent_digest()
    );
    assert_eq!(
        replay_mismatch_matrix.expected_replay_digest(),
        replay_mismatch_matrix.expected_bundle().digest()
    );
    assert_eq!(
        replay_mismatch_matrix.replayed_replay_digest(),
        replay_mismatch_matrix.replayed_bundle().digest()
    );
    assert_eq!(
        replay_mismatch_matrix.expected_semantic_digest(),
        replay_mismatch_matrix.expected_bundle().semantic_digest()
    );
    assert_eq!(
        replay_mismatch_matrix.replayed_semantic_digest(),
        replay_mismatch_matrix.replayed_bundle().semantic_digest()
    );
    assert_eq!(
        replay_mismatch_matrix.expected_effect_intent_digest(),
        replay_mismatch_matrix
            .expected_bundle()
            .effect_intent_digest()
    );
    assert_eq!(
        replay_mismatch_matrix.replayed_effect_intent_digest(),
        replay_mismatch_matrix
            .replayed_bundle()
            .effect_intent_digest()
    );
    assert_ne!(
        replay_mismatch_matrix.expected_effect_intent_patch_canonical_basis(),
        replay_mismatch_matrix.replayed_effect_intent_patch_canonical_basis()
    );
    assert_eq!(
        replay_mismatch_matrix.expected_effect_intent_patch_canonical_basis(),
        replay_mismatch_matrix
            .expected_bundle()
            .effect_intent_patch_canonical_basis()
    );
    assert_eq!(
        replay_mismatch_matrix.replayed_effect_intent_patch_canonical_basis(),
        replay_mismatch_matrix
            .replayed_bundle()
            .effect_intent_patch_canonical_basis()
    );
    assert_eq!(
        replay_mismatch_matrix.rebuilt_replay_digest(),
        replay_mismatch_matrix.rebuilt_bundle().digest()
    );
    assert_eq!(
        replay_mismatch_matrix.rebuilt_effect_intent_digest(),
        replay_mismatch_matrix
            .rebuilt_bundle()
            .effect_intent_digest()
    );
    assert_eq!(
        replay_mismatch_matrix.rebuilt_effect_intent_patch_canonical_basis(),
        replay_mismatch_matrix
            .rebuilt_bundle()
            .effect_intent_patch_canonical_basis()
    );
    assert_eq!(counter_snapshot.writeback_replay_request_count, 2);
    assert_eq!(counter_snapshot.writeback_replay_mismatch_count, 2);
}
