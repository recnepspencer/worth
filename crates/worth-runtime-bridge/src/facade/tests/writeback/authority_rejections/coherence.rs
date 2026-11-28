use super::super::support::*;

#[test]
fn runtime_rejects_idempotence_from_another_effect_before_authority_execution() {
    let runtime = runtime_with_writeback_authority(BridgeRuntimePolicy::default());
    let lowered_policy = lowered_policy(&runtime);
    let contract_a = runtime
        .admit_writeback_declaration(
            writeback_declaration(
                BridgeWritebackDeclarationIdentity::admit_bridge_owned("writeback:coherence-a"),
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                "coherence-a",
            ),
            &lowered_policy,
        )
        .expect("first writeback declaration should admit");
    let contract_b = runtime
        .admit_writeback_declaration(
            writeback_declaration_with_shape(
                BridgeWritebackDeclarationIdentity::admit_bridge_owned("writeback:coherence-b"),
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                BridgeWritebackEffectClass::AspectReconciliation,
                "coherence-b",
                BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
            ),
            &lowered_policy,
        )
        .expect("second writeback declaration should admit");
    let effect_a = runtime.lower_writeback_effect(
        &contract_a,
        &causality_basis(
            BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:coherence-a"),
            "commit-a",
        ),
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:coherence-a"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "coherence-a",
        ),
    );
    let effect_b = runtime.lower_writeback_effect(
        &contract_b,
        &causality_basis(
            BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:coherence-b"),
            "commit-b",
        ),
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:coherence-b"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::AspectReconciliation,
            "coherence-b",
        ),
    );
    let mismatched_idempotence = runtime.classify_writeback_idempotence(
        &effect_b,
        &lowered_policy,
        &truth_state_basis(&effect_b),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned("idempotence:coherence-b"),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    let error = runtime
        .execute_writeback_authority(&contract_a, &effect_a, &mismatched_idempotence)
        .expect_err("idempotence bound to another effect should fail before authority execution");

    assert_eq!(
        error.kind(),
        BridgeWritebackErrorKind::IdempotenceBasisMismatch
    );
}
