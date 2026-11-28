use super::super::support::*;

#[test]
fn runtime_rejects_receipt_with_mismatched_request_digest() {
    let runtime = runtime_with_custom_writeback_authority(
        BridgeRuntimePolicy::default(),
        MismatchedReceiptWritebackAuthority::default(),
    );
    let lowered_policy = lowered_policy(&runtime);
    let declaration = writeback_declaration(
        BridgeWritebackDeclarationIdentity::admit_bridge_owned("writeback:mismatched-receipt"),
        BridgeRequestKind::Authoritative,
        BridgeWritebackRequestMode::WritebackCapable,
        "mismatched-receipt",
    );
    let contract = runtime
        .admit_writeback_declaration(declaration, &lowered_policy)
        .expect("writeback declaration should admit");
    let causality = causality_basis(
        BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:mismatched-receipt"),
        "commit-a",
    );
    let effect = runtime.lower_writeback_effect(
        &contract,
        &causality,
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:mismatched-receipt"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "mismatched-receipt",
        ),
    );
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &truth_state_basis(&effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned("idempotence:mismatched-receipt"),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect("first request establishes a native prior receipt basis");

    let second_causality = causality_basis(
        BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:mismatched-receipt:second"),
        "commit-b",
    );
    let second_effect = runtime.lower_writeback_effect(
        &contract,
        &second_causality,
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:mismatched-receipt:second"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "mismatched-receipt:second",
        ),
    );
    let second_idempotence = runtime.classify_writeback_idempotence(
        &second_effect,
        &lowered_policy,
        &truth_state_basis(&second_effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:mismatched-receipt:second",
        ),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    let error = runtime
        .execute_writeback_authority(&contract, &second_effect, &second_idempotence)
        .expect_err("mismatched receipt request digests must fail closed");

    assert_eq!(error.kind(), BridgeWritebackErrorKind::InvariantRejected);
}

#[test]
fn runtime_rejects_rejected_receipt_without_failure_class() {
    let runtime = runtime_with_custom_writeback_authority(
        BridgeRuntimePolicy::default(),
        MalformedRejectedReceiptWritebackAuthority,
    );
    let lowered_policy = lowered_policy(&runtime);
    let declaration = writeback_declaration(
        BridgeWritebackDeclarationIdentity::admit_bridge_owned(
            "writeback:rejected-without-failure-class",
        ),
        BridgeRequestKind::Authoritative,
        BridgeWritebackRequestMode::WritebackCapable,
        "rejected-without-failure-class",
    );
    let contract = runtime
        .admit_writeback_declaration(declaration, &lowered_policy)
        .expect("writeback declaration should admit");
    let causality = causality_basis(
        BridgeWritebackCausalityIdentity::admit_bridge_owned(
            "causality:rejected-without-failure-class",
        ),
        "commit-a",
    );
    let effect = runtime.lower_writeback_effect(
        &contract,
        &causality,
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:rejected-without-failure-class"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "rejected-without-failure-class",
        ),
    );
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &truth_state_basis(&effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:rejected-without-failure-class",
        ),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    let error = runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect_err("rejected receipts without failure class must fail closed");

    assert_eq!(error.kind(), BridgeWritebackErrorKind::InvariantRejected);
}

#[test]
fn runtime_rejects_successful_receipt_with_failure_class() {
    let runtime = runtime_with_custom_writeback_authority(
        BridgeRuntimePolicy::default(),
        MalformedSuccessfulReceiptWritebackAuthority,
    );
    let lowered_policy = lowered_policy(&runtime);
    let declaration = writeback_declaration(
        BridgeWritebackDeclarationIdentity::admit_bridge_owned(
            "writeback:success-with-failure-class",
        ),
        BridgeRequestKind::Authoritative,
        BridgeWritebackRequestMode::WritebackCapable,
        "success-with-failure-class",
    );
    let contract = runtime
        .admit_writeback_declaration(declaration, &lowered_policy)
        .expect("writeback declaration should admit");
    let causality = causality_basis(
        BridgeWritebackCausalityIdentity::admit_bridge_owned(
            "causality:success-with-failure-class",
        ),
        "commit-a",
    );
    let effect = runtime.lower_writeback_effect(
        &contract,
        &causality,
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:success-with-failure-class"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "success-with-failure-class",
        ),
    );
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &truth_state_basis(&effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:success-with-failure-class",
        ),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    let error = runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect_err("successful receipts with failure class must fail closed");

    assert_eq!(error.kind(), BridgeWritebackErrorKind::InvariantRejected);
}
