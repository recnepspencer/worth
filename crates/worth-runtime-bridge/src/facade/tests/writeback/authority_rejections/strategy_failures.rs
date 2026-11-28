use super::super::support::*;

#[test]
fn runtime_maps_typed_authority_rejection_into_bridge_error_kind() {
    let runtime = runtime_with_custom_writeback_authority(
        BridgeRuntimePolicy::default(),
        RejectingWritebackAuthority {
            failure_class: BridgeWritebackFailureClass::StaleTruthBasis,
        },
    );
    let lowered_policy = lowered_policy(&runtime);
    let declaration = writeback_declaration(
        BridgeWritebackDeclarationIdentity::admit_bridge_owned("writeback:typed-rejection"),
        BridgeRequestKind::Authoritative,
        BridgeWritebackRequestMode::WritebackCapable,
        "typed-rejection",
    );
    let contract = runtime
        .admit_writeback_declaration(declaration, &lowered_policy)
        .expect("writeback declaration should admit");
    let causality = causality_basis(
        BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:typed-rejection"),
        "commit-a",
    );
    let effect = runtime.lower_writeback_effect(
        &contract,
        &causality,
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:typed-rejection"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "typed-rejection",
        ),
    );
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &truth_state_basis(&effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned("idempotence:typed-rejection"),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    let error = runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect_err("typed authority rejection should surface as bridge error");

    assert_eq!(error.kind(), BridgeWritebackErrorKind::StaleTruthBasis);
}

#[test]
fn runtime_maps_authority_transport_failure_into_strategy_failed() {
    let runtime = runtime_with_custom_writeback_authority(
        BridgeRuntimePolicy::default(),
        FailingWritebackAuthority,
    );
    let lowered_policy = lowered_policy(&runtime);
    let declaration = writeback_declaration(
        BridgeWritebackDeclarationIdentity::admit_bridge_owned("writeback:transport-failure"),
        BridgeRequestKind::Authoritative,
        BridgeWritebackRequestMode::WritebackCapable,
        "transport-failure",
    );
    let contract = runtime
        .admit_writeback_declaration(declaration, &lowered_policy)
        .expect("writeback declaration should admit");
    let causality = causality_basis(
        BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:transport-failure"),
        "commit-a",
    );
    let effect = runtime.lower_writeback_effect(
        &contract,
        &causality,
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:transport-failure"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "transport-failure",
        ),
    );
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &truth_state_basis(&effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned("idempotence:transport-failure"),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    let error = runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect_err("authority transport failure should surface as bridge error");

    assert_eq!(error.kind(), BridgeWritebackErrorKind::StrategyFailed);
}

#[test]
fn runtime_maps_authority_panic_into_strategy_panicked() {
    let runtime = runtime_with_custom_writeback_authority(
        BridgeRuntimePolicy::default(),
        PanickingWritebackAuthority,
    );
    let lowered_policy = lowered_policy(&runtime);
    let declaration = writeback_declaration(
        BridgeWritebackDeclarationIdentity::admit_bridge_owned("writeback:panic-failure"),
        BridgeRequestKind::Authoritative,
        BridgeWritebackRequestMode::WritebackCapable,
        "panic-failure",
    );
    let contract = runtime
        .admit_writeback_declaration(declaration, &lowered_policy)
        .expect("writeback declaration should admit");
    let causality = causality_basis(
        BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:panic-failure"),
        "commit-a",
    );
    let effect = runtime.lower_writeback_effect(
        &contract,
        &causality,
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:panic-failure"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "panic-failure",
        ),
    );
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &truth_state_basis(&effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned("idempotence:panic-failure"),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    let error = runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect_err("authority panic should surface as typed bridge error");

    assert_eq!(error.kind(), BridgeWritebackErrorKind::StrategyPanicked);
}
