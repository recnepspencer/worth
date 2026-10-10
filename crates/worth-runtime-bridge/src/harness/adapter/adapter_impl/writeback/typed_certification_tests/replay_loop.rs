use crate::writeback::BridgeWritebackLoopDisposition;

use super::*;

#[test]
fn replay_loop_certification_retains_typed_isolation_proof() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let WritebackHarnessExecution::CrossFamilyReplayLoopIsolationCertification {
        replay_loop_matrix,
        counter_snapshot,
        ..
    } = certified_execution(
        WritebackHarnessTarget::CrossFamilyReplayLoopIsolationCertification,
        resource_request,
    )
    else {
        panic!("replay loop should produce typed matrix");
    };

    let projected_family = replay_loop_matrix.projected_family();
    let aspect_family = replay_loop_matrix.aspect_family();
    let cross_family_replay = replay_loop_matrix.cross_family_replay_isolation();
    let same_family_equivalence = replay_loop_matrix.same_family_equivalence();
    let changed_causality = replay_loop_matrix.same_family_changed_causality();
    let loop_isolation = replay_loop_matrix.cross_family_loop_isolation();

    assert_eq!(
        projected_family.writeback_effect_artifact_digest(),
        projected_family.effect().digest()
    );
    assert_eq!(
        projected_family.effect_intent_digest(),
        projected_family.effect().effect_intent_digest()
    );
    assert_eq!(
        projected_family.effect_intent_patch_canonical_basis(),
        projected_family
            .effect()
            .effect_intent()
            .patch_canonical_basis()
    );
    assert_eq!(
        projected_family.mapped_input_digest(),
        projected_family.effect().mapped_input_digest()
    );
    assert_eq!(
        projected_family.causality_digest(),
        projected_family.effect().causality_digest()
    );
    assert_eq!(
        projected_family.idempotence_digest(),
        projected_family.idempotence().digest()
    );
    assert_eq!(
        projected_family.replay_bundle_digest(),
        projected_family.replay_bundle().digest()
    );
    assert_eq!(
        projected_family.replay_semantic_digest(),
        projected_family.replay_bundle().semantic_digest()
    );
    assert_eq!(
        aspect_family.effect_intent_digest(),
        aspect_family.effect().effect_intent_digest()
    );
    assert_eq!(
        aspect_family.replay_bundle_digest(),
        aspect_family.replay_bundle().digest()
    );
    assert_ne!(
        projected_family.replay_bundle_digest(),
        aspect_family.replay_bundle_digest()
    );

    assert!(cross_family_replay.semantic_digest_separated());
    assert_ne!(
        cross_family_replay.projected_bundle().semantic_digest(),
        cross_family_replay.aspect_bundle().semantic_digest()
    );
    assert_eq!(
        cross_family_replay.failure_kind(),
        cross_family_replay.error().kind()
    );
    assert_eq!(
        cross_family_replay.family_replay_record_digest(),
        cross_family_replay.replay_record().digest()
    );
    assert!(same_family_equivalence.semantic_digest_equal());
    assert_eq!(
        same_family_equivalence.projected_bundle().semantic_digest(),
        same_family_equivalence
            .rebuilt_projected_bundle()
            .semantic_digest()
    );
    assert_eq!(
        same_family_equivalence.effect_intent_digest_equal(),
        same_family_equivalence
            .projected_effect()
            .effect_intent_digest()
            == same_family_equivalence
                .rebuilt_projected_effect()
                .effect_intent_digest()
    );
    assert_eq!(
        same_family_equivalence.family_execution_record_digest(),
        same_family_equivalence.rebuilt_execution_record().digest()
    );
    assert!(changed_causality.semantic_digest_separated());
    assert_ne!(
        changed_causality.projected_bundle().semantic_digest(),
        changed_causality
            .changed_projected_bundle()
            .semantic_digest()
    );
    assert_eq!(
        changed_causality.failure_kind(),
        changed_causality.error().kind()
    );
    assert_eq!(
        changed_causality.family_replay_record_digest(),
        changed_causality.replay_record().digest()
    );
    assert_eq!(
        loop_isolation.incoming_feedback_provenance_digest(),
        loop_isolation
            .incoming_feedback_context()
            .provenance_digest()
    );
    assert_eq!(
        loop_isolation.incoming_feedback_causality_digest(),
        loop_isolation
            .incoming_feedback_context()
            .causality_digest()
    );
    assert_eq!(
        loop_isolation.digest(),
        loop_isolation.loop_prevention().digest()
    );
    assert_eq!(
        loop_isolation.disposition(),
        BridgeWritebackLoopDisposition::RejectAsUnsafeFeedback
    );
    assert_eq!(counter_snapshot.writeback_decision_record_append_count, 11);
}
