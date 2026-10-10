use super::*;

#[test]
fn feedback_certification_retains_typed_convergence_proof() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let WritebackHarnessExecution::FeedbackLoopCertification {
        feedback_loop_digest,
        feedback_route_identity,
        feedback_origin_matrix,
        counter_snapshot,
    } = certified_execution(
        WritebackHarnessTarget::FeedbackLoopCertification,
        resource_request,
    )
    else {
        panic!("feedback certification should produce feedback-loop typed matrix");
    };

    assert!(!feedback_loop_digest.is_empty());
    assert!(!feedback_route_identity.as_str().is_empty());
    assert!(!feedback_origin_matrix.feedback_route_digest().is_empty());
    assert_eq!(
        feedback_origin_matrix.feedback_route_identity(),
        &feedback_route_identity
    );
    assert_eq!(
        feedback_origin_matrix.causality_digest(),
        feedback_origin_matrix.causality().digest()
    );
    assert_eq!(
        feedback_origin_matrix.feedback_provenance_digest(),
        feedback_origin_matrix.feedback_provenance().digest()
    );
    assert_eq!(
        feedback_origin_matrix.carried_causality_digest(),
        feedback_origin_matrix
            .carried_feedback_context()
            .causality_digest()
    );
    assert_eq!(
        feedback_origin_matrix.carried_feedback_provenance_digest(),
        feedback_origin_matrix
            .carried_feedback_context()
            .provenance_digest()
    );
    assert_eq!(
        feedback_origin_matrix.writeback_digest(),
        feedback_origin_matrix.replay_digest()
    );
    assert_eq!(
        feedback_origin_matrix.writeback_effect_artifact_digest(),
        feedback_origin_matrix.effect().digest()
    );
    assert_eq!(
        feedback_origin_matrix.effect_intent_digest(),
        feedback_origin_matrix.effect().effect_intent_digest()
    );
    assert_eq!(
        feedback_origin_matrix.effect_intent_patch_canonical_basis(),
        feedback_origin_matrix
            .effect()
            .effect_intent()
            .patch_canonical_basis()
    );
    assert_eq!(
        feedback_origin_matrix.mutation_plan_digest(),
        feedback_origin_matrix
            .initial_outcome()
            .authoritative_artifact_digest()
    );

    let replay_bundle_report = feedback_origin_matrix.replay_bundle_report();
    assert_eq!(
        replay_bundle_report.digest(),
        replay_bundle_report.replay_bundle().digest()
    );
    assert_eq!(
        replay_bundle_report.effect_intent_digest(),
        replay_bundle_report.replay_bundle().effect_intent_digest()
    );
    assert_eq!(
        replay_bundle_report.effect_intent_patch_canonical_basis(),
        replay_bundle_report
            .replay_bundle()
            .effect_intent_patch_canonical_basis()
    );
    assert_eq!(
        replay_bundle_report.causality_digest(),
        replay_bundle_report.replay_bundle().causality_digest()
    );
    assert_eq!(
        replay_bundle_report.outcome_class(),
        crate::writeback::BridgeWritebackOutcomeClass::CanonicalNoop
    );

    let idempotence_report = feedback_origin_matrix.idempotence_report();
    assert_eq!(
        idempotence_report.initial_digest(),
        idempotence_report.initial_idempotence().digest()
    );
    assert_eq!(
        idempotence_report.replayed_digest(),
        idempotence_report.replayed_idempotence().digest()
    );
    assert_eq!(
        idempotence_report.initial_authoritative_state_digest(),
        idempotence_report
            .initial_idempotence()
            .authoritative_state_digest()
    );
    assert_eq!(
        idempotence_report.replayed_authoritative_state_digest(),
        idempotence_report
            .replayed_idempotence()
            .authoritative_state_digest()
    );
    assert_eq!(
        idempotence_report.lowered_policy_digest(),
        idempotence_report
            .initial_idempotence()
            .lowered_policy_digest()
    );

    let loop_prevention_report = feedback_origin_matrix.loop_prevention_report();
    assert_eq!(
        loop_prevention_report.digest(),
        loop_prevention_report.report().digest()
    );
    assert_eq!(
        loop_prevention_report.current_feedback_provenance_digest(),
        loop_prevention_report
            .report()
            .current_feedback_provenance_digest()
    );
    assert_eq!(
        loop_prevention_report.current_feedback_provenance_digest(),
        loop_prevention_report
            .report()
            .current_feedback_provenance()
            .digest()
    );
    assert_eq!(
        loop_prevention_report.current_causality_digest(),
        loop_prevention_report.report().current_causality_digest()
    );
    assert_eq!(
        loop_prevention_report.current_causality_digest(),
        loop_prevention_report
            .report()
            .current_feedback_provenance()
            .causality_digest()
    );
    assert_eq!(
        loop_prevention_report.report().idempotence_digest(),
        loop_prevention_report.report().idempotence().digest()
    );

    let authority_boundary = feedback_origin_matrix.authority_boundary_matrix();
    assert_eq!(
        authority_boundary.contract_digest(),
        authority_boundary.contract().digest()
    );
    assert_eq!(
        authority_boundary.strategy_coherence_digest(),
        authority_boundary.strategy_coherence().digest()
    );
    assert_eq!(
        authority_boundary.candidate_digest(),
        authority_boundary
            .candidate()
            .map(crate::writeback::BridgeValidatedWritebackCandidate::digest)
    );
    if let Some(authority_candidate) = authority_boundary.candidate() {
        assert_eq!(
            authority_candidate.effect_intent_digest(),
            feedback_origin_matrix.effect_intent_digest()
        );
    }
    assert_eq!(
        authority_boundary.authority_request_digest(),
        authority_boundary
            .authority_request()
            .map(crate::adapter::TruthWritebackRequest::digest)
    );
    assert_eq!(
        authority_boundary.authority_receipt_digest(),
        authority_boundary
            .authority_receipt()
            .map(crate::adapter::TruthWritebackReceipt::digest)
    );
    if let (Some(authority_request), Some(authority_receipt)) = (
        authority_boundary.authority_request(),
        authority_boundary.authority_receipt(),
    ) {
        assert_eq!(
            authority_request.effect_intent(),
            feedback_origin_matrix.effect().effect_intent()
        );
        assert_eq!(
            authority_request.effect_intent(),
            authority_receipt.effect_intent()
        );
    }

    let changed_effect_matrix = feedback_origin_matrix.changed_effect_feedback_matrix();
    assert_eq!(
        changed_effect_matrix.writeback_effect_artifact_digest(),
        changed_effect_matrix.changed_effect().digest()
    );
    assert_eq!(
        changed_effect_matrix.effect_intent_digest(),
        changed_effect_matrix
            .changed_effect()
            .effect_intent_digest()
    );
    assert_eq!(
        changed_effect_matrix.idempotence_digest(),
        changed_effect_matrix.changed_idempotence().digest()
    );
    assert_eq!(
        changed_effect_matrix.failure_kind(),
        changed_effect_matrix.failure().kind()
    );
    assert!(changed_effect_matrix.same_causality_as_initial());
    assert!(!changed_effect_matrix.same_feedback_provenance_as_initial());

    let interleaved_truth = feedback_origin_matrix.interleaved_truth_matrix();
    assert_eq!(
        interleaved_truth.ordinary_truth_commit_identity(),
        interleaved_truth.ordinary_truth_commit().as_str()
    );
    assert_eq!(
        interleaved_truth.bridge_feedback_commit_identity(),
        interleaved_truth.bridge_feedback_commit().as_str()
    );
    assert!(interleaved_truth
        .ordinary_truth_route_identity()
        .as_str()
        .starts_with("route:sha256:"));

    let restart_replay = feedback_origin_matrix.restart_replay_matrix();
    assert_eq!(
        restart_replay.rebuilt_contract_digest(),
        restart_replay.rebuilt_contract().digest()
    );
    assert_eq!(
        restart_replay.rebuilt_writeback_effect_artifact_digest(),
        restart_replay.rebuilt_effect().digest()
    );
    assert_eq!(
        restart_replay.rebuilt_effect_intent_digest(),
        restart_replay.rebuilt_effect().effect_intent_digest()
    );
    assert_eq!(
        restart_replay.rebuilt_idempotence_digest(),
        restart_replay.rebuilt_idempotence().digest()
    );
    assert_eq!(
        restart_replay.rebuilt_loop_prevention_digest(),
        restart_replay.rebuilt_loop_prevention().digest()
    );
    assert_eq!(
        restart_replay.rebuilt_outcome_digest(),
        restart_replay.rebuilt_outcome().digest()
    );
    assert_eq!(
        restart_replay.rebuilt_replay_bundle_digest(),
        restart_replay.rebuilt_replay_bundle().digest()
    );
    if let Some(rebuilt_receipt) = restart_replay.rebuilt_receipt() {
        assert_eq!(
            rebuilt_receipt.effect_intent(),
            restart_replay.rebuilt_effect().effect_intent()
        );
    }
    assert!(restart_replay.replay_equivalent_to_live_feedback());

    let boundedness = feedback_origin_matrix.boundedness_proof();
    assert_eq!(boundedness.authoritative_commit_count(), 1);
    assert!(boundedness.feedback_publication_routed());
    assert!(boundedness.ordinary_truth_interleaved());
    assert!(boundedness.feedback_converged());
    assert!(boundedness.restart_replay_converged());
    assert_eq!(
        restart_replay.rebuilt_authority_receipt_present(),
        restart_replay.rebuilt_receipt().is_some()
    );
    assert_eq!(
        counter_snapshot.writeback_loop_prevention_rejection_count,
        1
    );
}
