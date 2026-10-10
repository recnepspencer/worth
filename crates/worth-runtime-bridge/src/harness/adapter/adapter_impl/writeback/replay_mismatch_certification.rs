use super::*;

pub(super) fn execute_replay_mismatch_certification(
    runtime: &crate::harness::adapter::BridgeHarnessSession,
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WritebackHarnessExecution, BridgeHarnessError> {
    let lowered_policy_bundle = lowered_policy(runtime_bridge)?;
    let declaration = crate::facade::BridgeWritebackDeclaration::writeback_capable(
        crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
            "harness:writeback-replay-mismatch",
        ),
        crate::facade::BridgeRequestKind::Authoritative,
        crate::facade::BridgeWritebackFamilyKind::ProjectedStateDiff,
        crate::facade::BridgeWritebackEffectClass::ProjectedStateDiff,
        crate::facade::BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let contract = runtime_bridge
        .admit_writeback_declaration(declaration, &lowered_policy_bundle)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback replay mismatch certification contract admission failed: {error}"
            ))
        })?;
    let causality = writeback_causality_basis(
        "harness:writeback-replay-mismatch-causality",
        "replay-mismatch",
        route_digest_for_first_patch(runtime_bridge, fixture, resource_request)?,
        "replay-mismatch",
        "replay-mismatch",
    );
    let expected_effect = runtime_bridge.lower_writeback_effect(
        &contract,
        &causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-replay-mismatch-effect:expected",
        ),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            contract.digest().to_owned(),
        ),
    );
    let replayed_effect = runtime_bridge.lower_writeback_effect(
        &contract,
        &causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-replay-mismatch-effect:replayed",
        ),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            expected_effect.digest().to_owned(),
        ),
    );
    let expected_idempotence = runtime_bridge.classify_writeback_idempotence(
        &expected_effect,
        &lowered_policy_bundle,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(&expected_effect),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-replay-mismatch-idempotence:expected",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let replayed_idempotence = runtime_bridge.classify_writeback_idempotence(
        &replayed_effect,
        &lowered_policy_bundle,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(&replayed_effect),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-replay-mismatch-idempotence:replayed",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let expected_outcome = execute_native_writeback_authority_outcome(
        runtime_bridge,
        &contract,
        &expected_effect,
        &expected_idempotence,
        "expected",
    )?;
    let replayed_outcome = execute_native_writeback_authority_outcome(
        runtime_bridge,
        &contract,
        &replayed_effect,
        &replayed_idempotence,
        "replayed",
    )?;
    let expected_bundle = runtime_bridge.replay_writeback_bundle(
        &contract,
        &expected_effect,
        &expected_idempotence,
        &expected_outcome,
    );
    let replayed_bundle = runtime_bridge.replay_writeback_bundle(
        &contract,
        &replayed_effect,
        &replayed_idempotence,
        &replayed_outcome,
    );
    let validation_error = runtime_bridge
        .validate_replayed_writeback_bundle(&expected_bundle, &replayed_bundle)
        .expect_err("writeback replay mismatch certification must fail on semantic drift");
    let rebuilt_runtime = build_writeback_runtime(runtime, fixture, true)?;
    let rebuilt_lowered_policy_bundle = lowered_policy(&rebuilt_runtime)?;
    let rebuilt_contract = rebuilt_runtime
        .admit_writeback_declaration(
            crate::facade::BridgeWritebackDeclaration::writeback_capable(
                crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "harness:writeback-replay-mismatch",
                ),
                crate::facade::BridgeRequestKind::Authoritative,
                crate::facade::BridgeWritebackFamilyKind::ProjectedStateDiff,
                crate::facade::BridgeWritebackEffectClass::ProjectedStateDiff,
                crate::facade::BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
                crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
            ),
            &rebuilt_lowered_policy_bundle,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback replay mismatch certification rebuilt contract admission failed: {error}"
            ))
        })?;
    let rebuilt_replayed_effect = rebuilt_runtime.lower_writeback_effect(
        &rebuilt_contract,
        &causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-replay-mismatch-effect:replayed",
        ),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            expected_effect.digest().to_owned(),
        ),
    );
    let rebuilt_replayed_idempotence = rebuilt_runtime.classify_writeback_idempotence(
        &rebuilt_replayed_effect,
        &rebuilt_lowered_policy_bundle,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(
            &rebuilt_replayed_effect,
        ),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-replay-mismatch-idempotence:replayed",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let rebuilt_replayed_outcome = execute_native_writeback_authority_outcome(
        &rebuilt_runtime,
        &rebuilt_contract,
        &rebuilt_replayed_effect,
        &rebuilt_replayed_idempotence,
        "rebuilt replayed",
    )?;
    let rebuilt_replayed_bundle = rebuilt_runtime.replay_writeback_bundle(
        &rebuilt_contract,
        &rebuilt_replayed_effect,
        &rebuilt_replayed_idempotence,
        &rebuilt_replayed_outcome,
    );
    let rebuilt_validation_error = rebuilt_runtime
        .validate_replayed_writeback_bundle(&expected_bundle, &rebuilt_replayed_bundle)
        .expect_err(
            "writeback replay mismatch certification must fail on semantic drift after rebuild",
        );
    let counters = aggregate_runtime_writeback_counters(&[runtime_bridge, &rebuilt_runtime]);
    let counter_snapshot = snapshot_from_counters(&counters);

    Ok(WritebackHarnessExecution::ReplayMismatchCertification {
        replay_validation_digest: digest_string(
            "bridge-writeback-replay-validation",
            &format!(
                "expected={}|replayed={}|failure={:?}",
                expected_bundle.semantic_digest(),
                replayed_bundle.semantic_digest(),
                validation_error.kind()
            ),
        )
        .to_string(),
        replay_mismatch_matrix: WritebackReplayMismatchMatrix::from_replay_validation(
            &expected_bundle,
            &replayed_bundle,
            &validation_error,
            &rebuilt_replayed_bundle,
            &rebuilt_validation_error,
        ),
        counter_snapshot,
    })
}

fn execute_native_writeback_authority_outcome(
    runtime_bridge: &crate::facade::RuntimeBridge,
    contract: &crate::facade::AdmittedBridgeWritebackContract,
    effect: &crate::facade::BridgeDerivedWritebackEffect,
    idempotence: &crate::facade::BridgeWritebackIdempotenceBasis,
    authority_role: &str,
) -> Result<crate::facade::BridgeWritebackAuthorityOutcome, BridgeHarnessError> {
    runtime_bridge
        .execute_writeback_authority(contract, effect, idempotence)
        .map(|(outcome, _receipt)| outcome)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback replay mismatch certification {authority_role} authority execution failed: {error}"
            ))
        })
}
