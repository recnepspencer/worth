use crate::writeback::BridgeWritebackLoopDisposition;

use super::*;

#[test]
fn family_extension_certification_retains_typed_family_and_mapper_proof() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let WritebackHarnessExecution::ExtensibleFamilyCertification {
        family_extension_digest,
        family_extension_matrix,
        counter_snapshot,
    } = certified_execution(
        WritebackHarnessTarget::ExtensibleFamilyCertification,
        resource_request,
    )
    else {
        panic!("family extension should produce family typed matrix");
    };

    assert!(!family_extension_digest.is_empty());

    let projected_family = family_extension_matrix.projected_family();
    let aspect_family = family_extension_matrix.aspect_family();
    assert_eq!(
        projected_family.contract_digest(),
        projected_family.contract().digest()
    );
    assert_eq!(
        aspect_family.contract_digest(),
        aspect_family.contract().digest()
    );
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
        projected_family.authority_receipt_digest(),
        projected_family.authority_receipt().digest()
    );
    assert_eq!(
        projected_family.causality_digest(),
        aspect_family.causality_digest()
    );

    let cross_family_replay = family_extension_matrix.cross_family_replay_isolation();
    assert!(cross_family_replay.semantic_digest_separated());
    assert!(cross_family_replay.bundle_digest_separated());
    assert_eq!(
        cross_family_replay.family_replay_record_digest(),
        cross_family_replay.replay_record().digest()
    );
    assert_eq!(
        cross_family_replay.failure_kind(),
        cross_family_replay.error().kind()
    );
    assert_eq!(
        cross_family_replay.projected_bundle().digest(),
        projected_family.replay_bundle().digest()
    );
    assert_eq!(
        cross_family_replay.aspect_bundle().digest(),
        aspect_family.replay_bundle().digest()
    );

    let same_family_equivalence = family_extension_matrix.same_family_equivalence();
    assert!(same_family_equivalence.semantic_digest_equal());
    assert!(same_family_equivalence.bundle_digest_equal());
    assert!(same_family_equivalence.effect_intent_digest_equal());
    assert!(same_family_equivalence.mapped_input_digest_equal());
    assert_eq!(
        same_family_equivalence.family_execution_record_digest(),
        same_family_equivalence.rebuilt_execution_record().digest()
    );
    assert_eq!(
        same_family_equivalence
            .projected_effect()
            .effect_intent_digest(),
        same_family_equivalence
            .rebuilt_projected_effect()
            .effect_intent_digest()
    );
    assert_eq!(
        same_family_equivalence.projected_bundle().digest(),
        same_family_equivalence.rebuilt_projected_bundle().digest()
    );

    let changed_causality = family_extension_matrix.same_family_changed_causality();
    assert!(changed_causality.causality_digest_separated());
    assert!(changed_causality.semantic_digest_separated());
    assert!(changed_causality.bundle_digest_separated());
    assert_eq!(
        changed_causality.family_replay_record_digest(),
        changed_causality.replay_record().digest()
    );
    assert_eq!(
        changed_causality.failure_kind(),
        changed_causality.error().kind()
    );
    assert_eq!(
        changed_causality.projected_bundle().digest(),
        projected_family.replay_bundle().digest()
    );
    assert_ne!(
        changed_causality.projected_bundle().causality_digest(),
        changed_causality
            .changed_projected_bundle()
            .causality_digest()
    );

    let loop_isolation = family_extension_matrix.cross_family_loop_isolation();
    assert_eq!(
        loop_isolation.incoming_feedback_provenance_digest(),
        loop_isolation.feedback_context().provenance_digest()
    );
    assert_eq!(
        loop_isolation.incoming_feedback_causality_digest(),
        loop_isolation.feedback_context().causality_digest()
    );
    assert_eq!(
        loop_isolation.digest(),
        loop_isolation.loop_prevention().digest()
    );
    assert_eq!(
        loop_isolation.disposition(),
        BridgeWritebackLoopDisposition::RejectAsUnsafeFeedback
    );

    let mapper_parity = family_extension_matrix.mapper_parity_proof();
    assert!(mapper_parity.projected_mapper_envelope_retained());
    assert!(mapper_parity.aspect_mapper_envelope_retained());
    assert!(mapper_parity.projected_mapped_input_retained());
    assert!(mapper_parity.aspect_mapped_input_retained());
    assert_eq!(
        mapper_parity.projected_family_mapper_record_digest(),
        mapper_parity
            .projected_execution_record()
            .mapper_record_digest()
    );
    assert_eq!(
        mapper_parity.aspect_family_mapper_record_digest(),
        mapper_parity
            .aspect_execution_record()
            .mapper_record_digest()
    );
    assert_eq!(
        mapper_parity.projected_family_execution_record_digest(),
        mapper_parity.projected_execution_record().digest()
    );
    assert_eq!(
        mapper_parity.aspect_family_execution_record_digest(),
        mapper_parity.aspect_execution_record().digest()
    );

    let shadow_rejection = family_extension_matrix.shadow_protocol_rejection();
    assert_eq!(
        shadow_rejection.failure_kind(),
        crate::facade::BridgeWritebackErrorKind::FamilyBindingMismatch
    );
    assert_eq!(
        shadow_rejection.failure_kind(),
        shadow_rejection.error().kind()
    );
    assert!(shadow_rejection.effect_family_mismatch_rejected());
    assert!(shadow_rejection.no_shadow_protocol_mapper_envelope_retained());

    assert_eq!(counter_snapshot.writeback_commit_count, 4);
    assert_eq!(counter_snapshot.writeback_replay_mismatch_count, 2);
}
