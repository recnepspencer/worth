use super::support::*;

#[test]
fn writeback_batch_naming_digest_changes_with_attachment_identity() {
    let runtime = runtime_with_writeback_authority(BridgeRuntimePolicy::development());
    let lowered_policy = lowered_policy(&runtime);
    let contract = runtime
        .admit_writeback_declaration(
            writeback_declaration(
                BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "writeback:batch-mutation-authority-naming-digest",
                ),
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                "batch-mutation-authority-naming-digest",
            ),
            &lowered_policy,
        )
        .expect("writeback declaration should admit");

    let left = execute_bridge_mutation_bundle(
        &runtime,
        &lowered_policy,
        &contract,
        BridgeWritebackCausalityIdentity::admit_bridge_owned(
            "causality:batch-mutation-authority-naming-digest:left",
        ),
        "batch-mutation-authority-naming-digest:left",
        BridgeWritebackEffectIdentity::admit_bridge_owned(
            "effect:batch-mutation-authority-naming-digest:left",
        ),
        "batch-mutation-authority-naming-digest:left",
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:batch-mutation-authority-naming-digest:left",
        ),
    )
    .with_naming_mutation(
        crate::facade::BridgeNamingMutationBundle::attach_new_target(
            bridge_naming_attachment("persistent-name:left"),
            bridge_naming_target(
                crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts::entity(1, 3, 0),
            ),
            Some(bridge_naming_collection("Task")),
        ),
    );
    let right = execute_bridge_mutation_bundle(
        &runtime,
        &lowered_policy,
        &contract,
        BridgeWritebackCausalityIdentity::admit_bridge_owned(
            "causality:batch-mutation-authority-naming-digest:right",
        ),
        "batch-mutation-authority-naming-digest:right",
        BridgeWritebackEffectIdentity::admit_bridge_owned(
            "effect:batch-mutation-authority-naming-digest:right",
        ),
        "batch-mutation-authority-naming-digest:right",
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:batch-mutation-authority-naming-digest:right",
        ),
    )
    .with_naming_mutation(
        crate::facade::BridgeNamingMutationBundle::attach_new_target(
            bridge_naming_attachment("persistent-name:right"),
            bridge_naming_target(
                crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts::entity(1, 3, 0),
            ),
            Some(bridge_naming_collection("Task")),
        ),
    );

    let left_digest = crate::facade::BridgeBatchMutationAuthorityBundle::from_components(&[left])
        .expect("left component set should aggregate")
        .aggregate_naming_mutation_digest()
        .expect("left naming digest should exist")
        .to_string();
    let right_digest = crate::facade::BridgeBatchMutationAuthorityBundle::from_components(&[right])
        .expect("right component set should aggregate")
        .aggregate_naming_mutation_digest()
        .expect("right naming digest should exist")
        .to_string();

    assert_ne!(left_digest, right_digest);
}

fn bridge_naming_attachment(value: &str) -> crate::facade::BridgeNamingAttachmentIdentity {
    crate::facade::BridgeNamingAttachmentIdentity::from_bridge_evidence(
        &crate::facade::BridgeIdentityEvidence::from_bridge_owner_external_authority(value),
    )
}

fn bridge_naming_target(
    parts: crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts,
) -> crate::facade::BridgeNamingResolvedTargetIdentity {
    crate::facade::BridgeNamingResolvedTargetIdentity::from_relational_record(parts)
}

fn bridge_naming_collection(value: &str) -> crate::facade::BridgeNamingTargetCollection {
    crate::facade::BridgeNamingTargetCollection::new(value)
}

fn execute_bridge_mutation_bundle(
    runtime: &RuntimeBridge,
    lowered_policy: &crate::facade::LoweredBridgeExecutionPolicy,
    contract: &crate::facade::AdmittedBridgeWritebackContract,
    causality_identity: BridgeWritebackCausalityIdentity,
    truth_trigger_evidence_text: &str,
    effect_identity: BridgeWritebackEffectIdentity,
    effect_intent_value: &str,
    idempotence_identity: BridgeWritebackIdempotenceIdentity,
) -> crate::facade::BridgeMutationAuthorityBundle {
    let effect_intent = writeback_effect_intent(
        BridgeWritebackEffectClass::ProjectedStateDiff,
        effect_intent_value,
    );
    let causality = mutation_causality_basis(
        causality_identity,
        truth_trigger_evidence_text,
        &effect_intent,
    );
    let effect =
        runtime.lower_writeback_effect(contract, &causality, effect_identity, effect_intent);
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        lowered_policy,
        &truth_state_basis(&effect),
        idempotence_identity,
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    runtime
        .execute_writeback_mutation_authority(contract, &effect, &idempotence, &causality)
        .expect("matching writeback execution should mint mutation authority")
}
