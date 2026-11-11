use super::support::*;

#[test]
fn writeback_replay_comparison_retention_expires_after_a_rejected_semantic_drift() {
    let runtime = runtime_with_writeback_authority(
        BridgeRuntimePolicy::development().with_route_record_limit(1),
    );
    let lowered_policy = lowered_policy(&runtime);
    let declaration = writeback_declaration(
        BridgeWritebackDeclarationIdentity::admit_bridge_owned("writeback:replay-mismatch"),
        BridgeRequestKind::Authoritative,
        BridgeWritebackRequestMode::WritebackCapable,
        "replay-mismatch",
    );
    let contract = runtime
        .admit_writeback_declaration(declaration, &lowered_policy)
        .expect("writeback declaration should admit");
    let original_effect = runtime.lower_writeback_effect(
        &contract,
        &causality_basis(
            BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:replay-mismatch"),
            "commit-a",
        ),
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:replay-mismatch:original"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "replay-mismatch:original",
        ),
    );
    let drifted_effect = runtime.lower_writeback_effect(
        &contract,
        &causality_basis(
            BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:replay-mismatch"),
            "commit-a",
        ),
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:replay-mismatch:drifted"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "replay-mismatch:drifted",
        ),
    );
    let original_idempotence = runtime.classify_writeback_idempotence(
        &original_effect,
        &lowered_policy,
        &truth_state_basis(&original_effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:replay-mismatch:original",
        ),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let drifted_idempotence = runtime.classify_writeback_idempotence(
        &drifted_effect,
        &lowered_policy,
        &truth_state_basis(&drifted_effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:replay-mismatch:drifted",
        ),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    let original_outcome =
        execute_native_commit_outcome(&runtime, &contract, &original_effect, &original_idempotence);
    let drifted_outcome =
        execute_native_commit_outcome(&runtime, &contract, &drifted_effect, &drifted_idempotence);
    let original_bundle = runtime.replay_writeback_bundle(
        &contract,
        &original_effect,
        &original_idempotence,
        &original_outcome,
    );
    let drifted_bundle = runtime.replay_writeback_bundle(
        &contract,
        &drifted_effect,
        &drifted_idempotence,
        &drifted_outcome,
    );

    let error = runtime
        .validate_replayed_writeback_bundle(&original_bundle, &drifted_bundle)
        .expect_err("replayed writeback bundle should reject semantic drift");

    assert_eq!(error.kind(), BridgeWritebackErrorKind::ReplayMismatch);
    assert_ne!(original_bundle.digest(), drifted_bundle.digest());
    assert_ne!(
        original_bundle.semantic_digest(),
        drifted_bundle.semantic_digest()
    );

    let replay_record = runtime
        .diagnostics()
        .last_writeback_replay_record()
        .expect("runtime should retain a native writeback replay record");
    assert_eq!(
        replay_record.failure_class(),
        Some(crate::facade::BridgeWritebackFailureClass::ReplayMismatch)
    );
    assert_eq!(
        replay_record.expected_replay_digest(),
        original_bundle.digest()
    );
    assert_eq!(
        replay_record.replayed_replay_digest(),
        drifted_bundle.digest()
    );
    assert_eq!(
        replay_record.expected_effect_intent_digest(),
        original_bundle.effect_intent_digest()
    );
    assert_eq!(
        replay_record.replayed_effect_intent_digest(),
        drifted_bundle.effect_intent_digest()
    );
    assert_eq!(
        replay_record.expected_effect_intent_patch_canonical_basis(),
        original_bundle.effect_intent_patch_canonical_basis()
    );
    assert_eq!(
        replay_record.replayed_effect_intent_patch_canonical_basis(),
        drifted_bundle.effect_intent_patch_canonical_basis()
    );
    assert_eq!(replay_record.counters().writeback_replay_request_count(), 1);
    assert_eq!(
        replay_record.counters().writeback_replay_mismatch_count(),
        1
    );

    let replay_record_explanation = runtime
        .diagnostics()
        .explain_last_writeback_replay_record()
        .expect("writeback replay record explanation should exist");
    assert_eq!(replay_record_explanation.replay_record(), &replay_record);
    assert_eq!(
        replay_record_explanation.expected_causality_digest(),
        original_bundle.causality_digest()
    );
    assert_eq!(
        replay_record_explanation.replayed_causality_digest(),
        drifted_bundle.causality_digest()
    );
    assert_eq!(
        replay_record_explanation.expected_effect_intent_digest(),
        original_bundle.effect_intent_digest()
    );
    assert_eq!(
        replay_record_explanation.replayed_effect_intent_digest(),
        drifted_bundle.effect_intent_digest()
    );
    assert_eq!(
        replay_record_explanation.expected_effect_intent_patch_canonical_basis(),
        original_bundle.effect_intent_patch_canonical_basis()
    );
    assert_eq!(
        replay_record_explanation.replayed_effect_intent_patch_canonical_basis(),
        drifted_bundle.effect_intent_patch_canonical_basis()
    );
    let reference = runtime
        .diagnostics()
        .retained_writeback_replay_reference_for_identity(replay_record.record_identity())
        .unwrap();
    assert_eq!(
        runtime
            .diagnostics()
            .writeback_replay_record_for_reference(&reference)
            .unwrap(),
        replay_record
    );
    runtime
        .validate_replayed_writeback_bundle(&original_bundle, &original_bundle)
        .unwrap();
    super::diagnostic_retention::comparison_expiry(
        &runtime.diagnostics.handle(),
        &reference,
        &original_bundle,
    );
}

#[test]
fn runtime_accepts_replayed_writeback_bundle_when_only_diagnostics_detail_differs() {
    let standard_runtime = runtime_with_writeback_authority(BridgeRuntimePolicy::development());
    let exhaustive_runtime = runtime_with_writeback_authority(BridgeRuntimePolicy::forensic());
    let standard_lowered_policy = lowered_policy(&standard_runtime);
    let exhaustive_lowered_policy = lowered_policy(&exhaustive_runtime);

    let standard_contract = standard_runtime
        .admit_writeback_declaration(
            writeback_declaration(
                BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "writeback:replay-semantic-standard",
                ),
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                "replay-semantic",
            ),
            &standard_lowered_policy,
        )
        .expect("standard writeback declaration should admit");
    let exhaustive_contract = exhaustive_runtime
        .admit_writeback_declaration(
            writeback_declaration(
                BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "writeback:replay-semantic-exhaustive",
                ),
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                "replay-semantic",
            ),
            &exhaustive_lowered_policy,
        )
        .expect("exhaustive writeback declaration should admit");

    let standard_effect = standard_runtime.lower_writeback_effect(
        &standard_contract,
        &causality_basis(
            BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:replay-semantic"),
            "commit-a",
        ),
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:replay-semantic"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "replay-semantic",
        ),
    );
    let exhaustive_effect = exhaustive_runtime.lower_writeback_effect(
        &exhaustive_contract,
        &causality_basis(
            BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:replay-semantic"),
            "commit-a",
        ),
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:replay-semantic"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "replay-semantic",
        ),
    );
    let standard_idempotence = standard_runtime.classify_writeback_idempotence(
        &standard_effect,
        &standard_lowered_policy,
        &truth_state_basis(&standard_effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned("idempotence:replay-semantic"),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let exhaustive_idempotence = exhaustive_runtime.classify_writeback_idempotence(
        &exhaustive_effect,
        &exhaustive_lowered_policy,
        &truth_state_basis(&exhaustive_effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned("idempotence:replay-semantic"),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let standard_outcome = execute_native_commit_outcome(
        &standard_runtime,
        &standard_contract,
        &standard_effect,
        &standard_idempotence,
    );
    let exhaustive_outcome = execute_native_commit_outcome(
        &exhaustive_runtime,
        &exhaustive_contract,
        &exhaustive_effect,
        &exhaustive_idempotence,
    );
    let standard_bundle = standard_runtime.replay_writeback_bundle(
        &standard_contract,
        &standard_effect,
        &standard_idempotence,
        &standard_outcome,
    );
    let exhaustive_bundle = exhaustive_runtime.replay_writeback_bundle(
        &exhaustive_contract,
        &exhaustive_effect,
        &exhaustive_idempotence,
        &exhaustive_outcome,
    );

    standard_runtime
        .validate_replayed_writeback_bundle(&standard_bundle, &exhaustive_bundle)
        .expect("replay validation should accept diagnostics-only detail drift");
}
