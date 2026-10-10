use super::causal_envelope_mapping_support::{
    bridge_route_reference, bridge_writeback_admission_reference,
    bridge_writeback_execution_reference, bridge_writeback_mapped_input_reference,
    bridge_writeback_mapper_envelope_reference, bridge_writeback_mapper_record_reference,
    bridge_writeback_replay_reference, query_observation_reference,
};
use super::support::*;
use crate::facade::{
    BridgeCausalEnvelopeAssemblyRequest, BridgeCausalEvidenceReferenceIdentity,
    BridgeMappedWritebackFamilyInput, BridgeWritebackExecutionRecord,
    BridgeWritebackFamilyAdmissionRecord, BridgeWritebackMapperEnvelope,
    BridgeWritebackMapperRecord, BridgeWritebackReplayRecord, RuntimeBridge,
};

struct RetainedWritebackChain {
    admission: BridgeWritebackFamilyAdmissionRecord,
    mapper_envelope: BridgeWritebackMapperEnvelope,
    mapped_input: BridgeMappedWritebackFamilyInput,
    mapper_record: BridgeWritebackMapperRecord,
    execution: BridgeWritebackExecutionRecord,
    replay: BridgeWritebackReplayRecord,
}

struct RetainedWritebackChainInput {
    declaration_identity: BridgeWritebackDeclarationIdentity,
    strategy_descriptor_evidence_text: String,
    causality_identity: BridgeWritebackCausalityIdentity,
    causality_evidence_text: String,
    effect_identity: BridgeWritebackEffectIdentity,
    effect_intent_value: String,
    idempotence_identity: BridgeWritebackIdempotenceIdentity,
    drifted_effect_identity: BridgeWritebackEffectIdentity,
    drifted_effect_intent_value: String,
    drifted_idempotence_identity: BridgeWritebackIdempotenceIdentity,
}

#[test]
fn causal_envelope_full_writeback_chain_lookup_cost_ignores_unrelated_records() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut envelope_identities = Vec::new();

    for unrelated_records in [0, 2, 5] {
        let runtime = runtime_with_writeback_authority(BridgeRuntimePolicy::development());
        for index in 0..unrelated_records {
            let suffix = format!("noise-{index}");
            retain_writeback_chain(
                &runtime,
                RetainedWritebackChainInput {
                    declaration_identity: BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                        format!("writeback:causal-scale-{suffix}"),
                    ),
                    strategy_descriptor_evidence_text: format!("causal-scale-{suffix}"),
                    causality_identity: BridgeWritebackCausalityIdentity::admit_bridge_owned(
                        format!("causality:causal-scale-{suffix}"),
                    ),
                    causality_evidence_text: format!("causal-scale-{suffix}"),
                    effect_identity: BridgeWritebackEffectIdentity::admit_bridge_owned(format!(
                        "effect:causal-scale-{suffix}"
                    )),
                    effect_intent_value: format!("causal-scale-{suffix}"),
                    idempotence_identity: BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
                        format!("idempotence:causal-scale-{suffix}"),
                    ),
                    drifted_effect_identity: BridgeWritebackEffectIdentity::admit_bridge_owned(
                        format!("effect:causal-scale-{suffix}-drifted"),
                    ),
                    drifted_effect_intent_value: format!("causal-scale-{suffix}-drifted"),
                    drifted_idempotence_identity:
                        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(format!(
                            "idempotence:causal-scale-{suffix}-drifted"
                        )),
                },
            );
        }
        let target_chain = retain_writeback_chain(
            &runtime,
            RetainedWritebackChainInput {
                declaration_identity: BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "writeback:causal-scale-target",
                ),
                strategy_descriptor_evidence_text: "causal-scale-target".to_string(),
                causality_identity: BridgeWritebackCausalityIdentity::admit_bridge_owned(
                    "causality:causal-scale-target",
                ),
                causality_evidence_text: "causal-scale-target".to_string(),
                effect_identity: BridgeWritebackEffectIdentity::admit_bridge_owned(
                    "effect:causal-scale-target",
                ),
                effect_intent_value: "causal-scale-target".to_string(),
                idempotence_identity: BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
                    "idempotence:causal-scale-target",
                ),
                drifted_effect_identity: BridgeWritebackEffectIdentity::admit_bridge_owned(
                    "effect:causal-scale-target-drifted",
                ),
                drifted_effect_intent_value: "causal-scale-target-drifted".to_string(),
                drifted_idempotence_identity:
                    BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
                        "idempotence:causal-scale-target-drifted",
                    ),
            },
        );
        let routed = runtime
            .route(
                crate::truth_identity_fixtures::truth_commit_fixture(
                    "commit-causal-writeback-full-scale",
                ),
                execution,
            )
            .expect("route should succeed");
        let request = BridgeCausalEnvelopeAssemblyRequest::from_query_admission(
            crate::facade::BridgeCausalInspectionAdmissionSummary::admitted(
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "query-admission:writeback-full-scale",
                ),
                crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                    "causal-anchor:writeback-full-scale",
                ),
            )
            .expect("query admission summary should be valid"),
            vec![
                query_observation_reference(
                    BridgeCausalEvidenceReferenceIdentity::query_observation(
                        crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(
                            "query-observation:writeback-full-scale",
                        ),
                    )
                    .expect("query observation reference identity should be valid"),
                ),
                bridge_route_reference(routed.result().result_summary()),
                bridge_writeback_admission_reference(&target_chain.admission),
                bridge_writeback_mapper_envelope_reference(&target_chain.mapper_envelope),
                bridge_writeback_mapped_input_reference(&target_chain.mapped_input),
                bridge_writeback_mapper_record_reference(&target_chain.mapper_record),
                bridge_writeback_execution_reference(&target_chain.execution),
                bridge_writeback_replay_reference(&target_chain.replay),
            ],
        )
        .expect("request should be valid");

        let envelope = runtime
            .diagnostics()
            .assemble_causal_explanation_envelope(request)
            .expect("full writeback chain should bind");

        assert_eq!(
            runtime.diagnostics().writeback_admission_records().len(),
            unrelated_records + 1
        );
        assert_eq!(
            runtime.diagnostics().writeback_mapper_envelopes().len(),
            (unrelated_records + 1) * 4
        );
        assert_eq!(
            runtime.diagnostics().writeback_mapped_family_inputs().len(),
            (unrelated_records + 1) * 3
        );
        assert_eq!(
            runtime.diagnostics().writeback_mapper_records().len(),
            unrelated_records + 1
        );
        assert_eq!(
            runtime.diagnostics().writeback_execution_records().len(),
            unrelated_records + 1
        );
        assert_eq!(
            runtime.diagnostics().writeback_replay_records().len(),
            unrelated_records + 1
        );
        assert_eq!(envelope.counters().bridge_retained_lookup_count(), 7);
        assert_eq!(envelope.counters().retained_bridge_binding_count(), 7);
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

fn retain_writeback_chain(
    runtime: &RuntimeBridge,
    input: RetainedWritebackChainInput,
) -> RetainedWritebackChain {
    let lowered_policy = lowered_policy(runtime);
    let contract = runtime
        .admit_writeback_declaration(
            writeback_declaration(
                input.declaration_identity,
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                &input.strategy_descriptor_evidence_text,
            ),
            &lowered_policy,
        )
        .expect("writeback declaration should admit");
    let admission = runtime
        .diagnostics()
        .last_writeback_admission_record()
        .expect("writeback admission should be retained");
    let causality = causality_basis(input.causality_identity, &input.causality_evidence_text);
    let mapper_envelope = runtime.lower_writeback_mapper_envelope(
        &contract,
        &causality,
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            input.effect_intent_value.clone(),
        ),
    );
    let mapped_input = runtime.map_writeback_family_input(
        &contract,
        &causality,
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            input.effect_intent_value.clone(),
        ),
    );
    let effect = runtime.lower_writeback_effect(
        &contract,
        &causality,
        input.effect_identity,
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            input.effect_intent_value,
        ),
    );
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &truth_state_basis(&effect),
        input.idempotence_identity,
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
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
    let expected_bundle =
        runtime.replay_writeback_bundle(&contract, &effect, &idempotence, &outcome);
    let drifted_effect = runtime.lower_writeback_effect(
        &contract,
        &causality,
        input.drifted_effect_identity,
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            input.drifted_effect_intent_value,
        ),
    );
    let drifted_idempotence = runtime.classify_writeback_idempotence(
        &drifted_effect,
        &lowered_policy,
        &truth_state_basis(&drifted_effect),
        input.drifted_idempotence_identity,
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let replayed_bundle =
        runtime.replay_writeback_bundle(&contract, &drifted_effect, &drifted_idempotence, &outcome);
    runtime
        .validate_replayed_writeback_bundle(&expected_bundle, &replayed_bundle)
        .expect_err("drifted replay should retain replay evidence");
    let replay = runtime
        .diagnostics()
        .last_writeback_replay_record()
        .expect("replay record should be retained");

    RetainedWritebackChain {
        admission,
        mapper_envelope,
        mapped_input,
        mapper_record,
        execution,
        replay,
    }
}
