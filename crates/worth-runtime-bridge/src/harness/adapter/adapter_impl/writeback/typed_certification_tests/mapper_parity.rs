use super::*;

#[test]
fn mapper_parity_certification_retains_typed_shadow_rejection_proof() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let WritebackHarnessExecution::HostMapperParityCertification {
        mapper_parity_matrix,
        counter_snapshot,
        ..
    } = certified_execution(
        WritebackHarnessTarget::HostMapperParityCertification,
        resource_request,
    )
    else {
        panic!("mapper parity should produce typed matrix");
    };

    let projected_family = mapper_parity_matrix.projected_family();
    let aspect_family = mapper_parity_matrix.aspect_family();

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
        projected_family.causality_digest(),
        projected_family.effect().causality_digest()
    );
    assert_eq!(
        projected_family.mapped_input_digest(),
        projected_family.effect().mapped_input_digest()
    );
    assert_eq!(
        projected_family.mapper_envelope_digest(),
        projected_family.effect().mapper_envelope_digest()
    );
    assert_eq!(
        projected_family.replay_bundle_digest(),
        projected_family.replay_bundle().digest()
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

    assert!(mapper_parity_matrix
        .mapper_parity_proof()
        .projected_mapper_envelope_retained());
    assert!(mapper_parity_matrix
        .mapper_parity_proof()
        .aspect_mapper_envelope_retained());
    assert_eq!(
        mapper_parity_matrix
            .shadow_protocol_rejection()
            .failure_kind(),
        crate::facade::BridgeWritebackErrorKind::FamilyBindingMismatch
    );
    assert!(mapper_parity_matrix
        .shadow_protocol_rejection()
        .effect_family_mismatch_rejected());
    assert_eq!(counter_snapshot.writeback_mapper_lowering_count, 2);
}
