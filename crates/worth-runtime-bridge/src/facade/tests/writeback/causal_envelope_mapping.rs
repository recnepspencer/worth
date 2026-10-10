use super::causal_envelope_mapping_support::{
    binding_for, bridge_route_reference, bridge_writeback_admission_reference,
    bridge_writeback_execution_reference, bridge_writeback_mapped_input_reference,
    bridge_writeback_mapper_envelope_reference, bridge_writeback_mapper_record_reference,
    bridge_writeback_replay_reference, missing_bridge_reference, query_observation_reference,
    writeback_admission_digest, writeback_execution_digest, writeback_mapped_input_digest,
    writeback_mapper_envelope_digest, writeback_mapper_record_digest, writeback_replay_digest,
};
use super::support::*;
use crate::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEnvelopeDenialKind,
    BridgeCausalEvidenceBindingClass, BridgeCausalEvidenceFamily,
    BridgeCausalEvidenceReferenceIdentity,
};

#[test]
fn causal_envelope_maps_retained_writeback_records_into_bridge_owned_bindings() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime_with_writeback_authority(BridgeRuntimePolicy::development());
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture("commit-causal-writeback"),
            execution,
        )
        .expect("route should succeed");
    let lowered_policy = lowered_policy(&runtime);
    let contract = runtime
        .admit_writeback_declaration(
            writeback_declaration(
                BridgeWritebackDeclarationIdentity::admit_bridge_owned("writeback:causal-envelope"),
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                "causal-envelope",
            ),
            &lowered_policy,
        )
        .expect("writeback declaration should admit");
    let admission = runtime
        .diagnostics()
        .last_writeback_admission_record()
        .expect("writeback admission should be retained");
    let causality = causality_basis(
        BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:causal-envelope"),
        "commit-a",
    );
    let mapper_envelope = runtime.lower_writeback_mapper_envelope(
        &contract,
        &causality,
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "causal-envelope",
        ),
    );
    let mapped_input = runtime.map_writeback_family_input(
        &contract,
        &causality,
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "causal-envelope",
        ),
    );
    let effect = runtime.lower_writeback_effect(
        &contract,
        &causality,
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:causal-envelope"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "causal-envelope",
        ),
    );
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &truth_state_basis(&effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned("idempotence:causal-envelope"),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let loop_prevention = runtime.classify_writeback_loop_prevention(&effect, &idempotence, None);
    let strategy_coherence =
        runtime.classify_writeback_strategy_coherence(&contract, &effect, &idempotence);
    let candidate = runtime
        .validate_writeback_candidate(
            &contract,
            &effect,
            &idempotence,
            &loop_prevention,
            &strategy_coherence,
        )
        .expect("candidate should validate");
    let (outcome, _) = runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect("authority execution should succeed");
    let mapper_record = runtime
        .diagnostics()
        .last_writeback_mapper_record()
        .expect("mapper record should be retained");
    let execution = runtime
        .diagnostics()
        .last_writeback_execution_record()
        .expect("execution record should be retained");
    let replay_bundle = runtime.replay_writeback_bundle(&contract, &effect, &idempotence, &outcome);
    let drifted = runtime.lower_writeback_effect(
        &contract,
        &causality,
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:causal-envelope-drifted"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "causal-envelope-drifted",
        ),
    );
    let drifted_idempotence = runtime.classify_writeback_idempotence(
        &drifted,
        &lowered_policy,
        &truth_state_basis(&drifted),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:causal-envelope-drifted",
        ),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let drifted_bundle =
        runtime.replay_writeback_bundle(&contract, &drifted, &drifted_idempotence, &outcome);
    runtime
        .validate_replayed_writeback_bundle(&replay_bundle, &drifted_bundle)
        .expect_err("drifted writeback replay should produce a retained replay record");
    let replay = runtime
        .diagnostics()
        .last_writeback_replay_record()
        .expect("replay record should be retained");

    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:writeback",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:writeback",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:writeback",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
            bridge_writeback_admission_reference(&admission),
            bridge_writeback_mapper_envelope_reference(&mapper_envelope),
            bridge_writeback_mapped_input_reference(&mapped_input),
            bridge_writeback_mapper_record_reference(&mapper_record),
            bridge_writeback_execution_reference(&execution),
            bridge_writeback_replay_reference(&replay),
        ],
    )
    .expect("request should be valid");

    let envelope = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect("writeback mappings should assemble");

    assert_eq!(envelope.bindings().len(), 8);
    assert_eq!(envelope.counters().bridge_retained_lookup_count(), 7);
    assert_eq!(envelope.counters().retained_bridge_binding_count(), 7);
    assert_eq!(envelope.counters().bridge_record_unindexed_scan_count(), 0);
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeWritebackAdmission,
            admission.record_identity().as_str()
        )
        .binding_class(),
        BridgeCausalEvidenceBindingClass::RetainedBridgeRecord
    );
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeWritebackAdmission,
            admission.record_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(writeback_admission_digest(&admission).as_str())
    );
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeWritebackMapperEnvelope,
            mapper_envelope.envelope_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(writeback_mapper_envelope_digest(&mapper_envelope).as_str())
    );
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeWritebackMappedFamilyInput,
            mapped_input.mapped_input_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(writeback_mapped_input_digest(&mapped_input).as_str())
    );
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeWritebackMapper,
            mapper_record.record_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(writeback_mapper_record_digest(&mapper_record).as_str())
    );
    assert_eq!(candidate.digest(), mapper_record.candidate_digest());
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeWritebackExecution,
            execution.record_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(writeback_execution_digest(&execution).as_str())
    );
    assert_eq!(
        binding_for(
            envelope.bindings(),
            BridgeCausalEvidenceFamily::BridgeWritebackReplay,
            replay.record_identity().as_str()
        )
        .retained_record_digest_for_reporting(),
        Some(writeback_replay_digest(&replay).as_str())
    );
}

#[test]
fn causal_envelope_denies_missing_writeback_replay_without_unindexed_scan() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let routed = runtime
        .route(
            crate::truth_identity_fixtures::truth_commit_fixture(
                "commit-causal-missing-writeback-replay",
            ),
            execution,
        )
        .expect("route should succeed");
    let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
        crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "query-admission:missing-writeback-replay",
            ),
            crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                "causal-anchor:missing-writeback-replay",
            ),
        )
        .expect("query admission summary should be valid"),
        vec![
            query_observation_reference(
                BridgeCausalEvidenceReferenceIdentity::query_observation(
                    crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                        "query-observation:missing-writeback-replay",
                    ),
                )
                .expect("query observation reference identity should be valid"),
            ),
            bridge_route_reference(routed.result().result_summary()),
            missing_bridge_reference(
                BridgeCausalEvidenceFamily::BridgeWritebackReplay,
                "missing-writeback-replay-record",
            ),
        ],
    )
    .expect("request should be valid");

    let denial = runtime
        .diagnostics()
        .assemble_causal_explanation_envelope(request)
        .expect_err("missing writeback replay record should deny");

    assert_eq!(
        denial.kind(),
        BridgeCausalEnvelopeDenialKind::MissingRetainedBridgeRecord
    );
    assert_eq!(
        denial.family(),
        BridgeCausalEvidenceFamily::BridgeWritebackReplay
    );
    assert_eq!(denial.counters().bridge_retained_lookup_count(), 2);
    assert_eq!(denial.counters().retained_bridge_binding_count(), 1);
    assert_eq!(denial.counters().missing_bridge_record_count(), 1);
    assert_eq!(denial.counters().bridge_record_unindexed_scan_count(), 0);
}

#[test]
fn causal_envelope_writeback_admission_lookup_cost_ignores_unrelated_records() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut envelope_identities = Vec::new();

    for unrelated_records in [0, 3, 8] {
        let runtime = runtime(BridgeRuntimePolicy::default());
        let lowered_policy = lowered_policy(&runtime);
        for index in 0..unrelated_records {
            runtime
                .admit_writeback_declaration(
                    writeback_declaration(
                        BridgeWritebackDeclarationIdentity::admit_bridge_owned(format!(
                            "writeback:noise-{index}"
                        )),
                        BridgeRequestKind::Authoritative,
                        BridgeWritebackRequestMode::WritebackCapable,
                        &format!("noise-{index}"),
                    ),
                    &lowered_policy,
                )
                .expect("noise writeback declaration should admit");
        }
        let target = runtime
            .admit_writeback_declaration(
                writeback_declaration(
                    BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                        "writeback:causal-scale",
                    ),
                    BridgeRequestKind::Authoritative,
                    BridgeWritebackRequestMode::WritebackCapable,
                    "causal-scale",
                ),
                &lowered_policy,
            )
            .expect("target writeback declaration should admit");
        let routed = runtime
            .route(
                crate::truth_identity_fixtures::truth_commit_fixture(
                    "commit-causal-writeback-scale",
                ),
                execution,
            )
            .expect("route should succeed");
        let target_record = runtime
            .diagnostics()
            .writeback_admission_record_for_contract_digest(target.digest())
            .expect("target admission record should be retained");
        let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
            crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "query-admission:writeback-scale",
                ),
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "causal-anchor:writeback-scale",
                ),
            )
            .expect("query admission summary should be valid"),
            vec![
                query_observation_reference(
                    BridgeCausalEvidenceReferenceIdentity::query_observation(
                        crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                            "query-observation:writeback-scale",
                        ),
                    )
                    .expect("query observation reference identity should be valid"),
                ),
                bridge_route_reference(routed.result().result_summary()),
                bridge_writeback_admission_reference(&target_record),
            ],
        )
        .expect("request should be valid");

        let envelope = runtime
            .diagnostics()
            .assemble_causal_explanation_envelope(request)
            .expect("target writeback admission should bind");

        assert_eq!(
            runtime.diagnostics().writeback_admission_records().len(),
            unrelated_records + 1
        );
        assert_eq!(envelope.counters().bridge_retained_lookup_count(), 2);
        assert_eq!(envelope.counters().retained_bridge_binding_count(), 2);
        assert_eq!(envelope.counters().bridge_record_unindexed_scan_count(), 0);
        envelope_identities.push(
            envelope
                .identity()
                .envelope_identity_for_reporting()
                .to_string(),
        );
    }

    assert_eq!(envelope_identities[0], envelope_identities[1]);
    assert_eq!(envelope_identities[1], envelope_identities[2]);
}
