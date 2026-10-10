use super::*;

pub(in crate::harness::adapter::adapter_impl::writeback) fn execute_cross_family_replay_loop_isolation_certification(
    runtime: &crate::harness::adapter::BridgeHarnessSession,
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WritebackHarnessExecution, BridgeHarnessError> {
    let lowered_policy_bundle = lowered_policy(runtime_bridge)?;
    let causality = writeback_causality_basis(
        "harness:writeback-family-replay-loop-isolation-causality",
        "family-replay-loop-isolation",
        route_digest_for_first_patch(runtime_bridge, fixture, resource_request)?,
        "family-replay-loop-isolation",
        "family-replay-loop-isolation",
    );
    let projected_contract = runtime_bridge
        .admit_writeback_declaration(
            crate::facade::BridgeWritebackDeclaration::writeback_capable(
                crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "harness:writeback-family-replay-loop-isolation:projected",
                ),
                crate::facade::BridgeRequestKind::Authoritative,
                crate::facade::BridgeWritebackFamilyKind::ProjectedStateDiff,
                crate::facade::BridgeWritebackEffectClass::ProjectedStateDiff,
                crate::facade::BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
                crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
            ),
            &lowered_policy_bundle,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "cross-family replay/loop isolation projected family admission failed: {error}"
            ))
        })?;
    let aspect_contract = runtime_bridge
        .admit_writeback_declaration(
            crate::facade::BridgeWritebackDeclaration::writeback_capable(
                crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "harness:writeback-family-replay-loop-isolation:aspect",
                ),
                crate::facade::BridgeRequestKind::Authoritative,
                crate::facade::BridgeWritebackFamilyKind::AspectReconciliation,
                crate::facade::BridgeWritebackEffectClass::AspectReconciliation,
                crate::facade::BridgeWritebackStrategyClass::AspectReconciliationCommit,
                crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
            ),
            &lowered_policy_bundle,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "cross-family replay/loop isolation aspect family admission failed: {error}"
            ))
        })?;
    let projected_effect = runtime_bridge.lower_writeback_effect(
        &projected_contract,
        &causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-family-replay-loop-isolation:effect:projected",
        ),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            projected_contract.digest().to_owned(),
        ),
    );
    let aspect_effect = runtime_bridge.lower_writeback_effect(
        &aspect_contract,
        &causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-family-replay-loop-isolation:effect:aspect",
        ),
        writeback_effect_intent(
            BridgeWritebackEffectClass::AspectReconciliation,
            aspect_contract.digest().to_owned(),
        ),
    );
    let projected_idempotence = runtime_bridge.classify_writeback_idempotence(
        &projected_effect,
        &lowered_policy_bundle,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(&projected_effect),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-family-replay-loop-isolation:idempotence:projected",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let aspect_idempotence = runtime_bridge.classify_writeback_idempotence(
        &aspect_effect,
        &lowered_policy_bundle,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(&aspect_effect),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-family-replay-loop-isolation:idempotence:aspect",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let (projected_outcome, _) = runtime_bridge
        .execute_writeback_authority(
            &projected_contract,
            &projected_effect,
            &projected_idempotence,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "cross-family replay/loop isolation projected execution failed: {error}"
            ))
        })?;
    let (aspect_outcome, _) = runtime_bridge
        .execute_writeback_authority(&aspect_contract, &aspect_effect, &aspect_idempotence)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "cross-family replay/loop isolation aspect execution failed: {error}"
            ))
        })?;
    let projected_bundle = runtime_bridge.replay_writeback_bundle(
        &projected_contract,
        &projected_effect,
        &projected_idempotence,
        &projected_outcome,
    );
    let aspect_bundle = runtime_bridge.replay_writeback_bundle(
        &aspect_contract,
        &aspect_effect,
        &aspect_idempotence,
        &aspect_outcome,
    );
    let projected_feedback = runtime_bridge.derive_writeback_feedback_provenance(&projected_effect);
    let projected_feedback_context =
        crate::facade::BridgeWritebackFeedbackContext::from_provenance(&projected_feedback);
    let cross_family_loop_prevention = runtime_bridge.classify_writeback_loop_prevention(
        &aspect_effect,
        &aspect_idempotence,
        Some(&projected_feedback_context),
    );

    let rebuilt_runtime = build_writeback_runtime_with_custom_authority(
        runtime,
        fixture,
        crate::harness::fixtures::RecordingTruthWritebackAuthority::default(),
    )?;
    let rebuilt_policy_bundle = lowered_policy(&rebuilt_runtime)?;
    let rebuilt_projected_contract = rebuilt_runtime
        .admit_writeback_declaration(
            crate::facade::BridgeWritebackDeclaration::writeback_capable(
                crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "harness:writeback-family-replay-loop-isolation:projected",
                ),
                crate::facade::BridgeRequestKind::Authoritative,
                crate::facade::BridgeWritebackFamilyKind::ProjectedStateDiff,
                crate::facade::BridgeWritebackEffectClass::ProjectedStateDiff,
                crate::facade::BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
                crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
            ),
            &rebuilt_policy_bundle,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "cross-family replay/loop isolation rebuilt projected admission failed: {error}"
            ))
        })?;
    let rebuilt_projected_effect = rebuilt_runtime.lower_writeback_effect(
        &rebuilt_projected_contract,
        &causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-family-replay-loop-isolation:effect:projected",
        ),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            rebuilt_projected_contract.digest().to_owned(),
        ),
    );
    let rebuilt_projected_idempotence = rebuilt_runtime.classify_writeback_idempotence(
        &rebuilt_projected_effect,
        &rebuilt_policy_bundle,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(
            &rebuilt_projected_effect,
        ),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-family-replay-loop-isolation:idempotence:projected",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let (rebuilt_projected_outcome, _) = rebuilt_runtime
        .execute_writeback_authority(
            &rebuilt_projected_contract,
            &rebuilt_projected_effect,
            &rebuilt_projected_idempotence,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "cross-family replay/loop isolation rebuilt projected execution failed: {error}"
            ))
        })?;
    let rebuilt_projected_bundle = rebuilt_runtime.replay_writeback_bundle(
        &rebuilt_projected_contract,
        &rebuilt_projected_effect,
        &rebuilt_projected_idempotence,
        &rebuilt_projected_outcome,
    );
    rebuilt_runtime
        .validate_replayed_writeback_bundle(&projected_bundle, &rebuilt_projected_bundle)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "cross-family replay/loop isolation same-family rebuilt validation unexpectedly failed: {error}"
            ))
        })?;
    let rebuilt_execution_record = rebuilt_runtime
        .diagnostics()
        .last_writeback_execution_record()
        .ok_or_else(|| {
            BridgeHarnessError::new(
                "cross-family replay/loop isolation rebuilt execution record missing",
            )
        })?;
    let changed_causality = writeback_causality_basis(
        "harness:writeback-family-replay-loop-isolation-causality:changed",
        causality.digest(),
        causality.digest(),
        "family-replay-loop-isolation",
        "family-replay-loop-isolation",
    );
    let changed_projected_effect = rebuilt_runtime.lower_writeback_effect(
        &rebuilt_projected_contract,
        &changed_causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-family-replay-loop-isolation:effect:projected:changed",
        ),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            rebuilt_projected_effect.digest().to_owned(),
        ),
    );
    let changed_projected_idempotence = rebuilt_runtime.classify_writeback_idempotence(
        &changed_projected_effect,
        &rebuilt_policy_bundle,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(
            &changed_projected_effect,
        ),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-family-replay-loop-isolation:idempotence:projected:changed",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let (changed_projected_outcome, _) = rebuilt_runtime
        .execute_writeback_authority(
            &rebuilt_projected_contract,
            &changed_projected_effect,
            &changed_projected_idempotence,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "cross-family replay/loop isolation changed-causality projected execution failed: {error}"
            ))
        })?;
    let changed_projected_bundle = rebuilt_runtime.replay_writeback_bundle(
        &rebuilt_projected_contract,
        &changed_projected_effect,
        &changed_projected_idempotence,
        &changed_projected_outcome,
    );
    let same_family_drift_error = rebuilt_runtime
        .validate_replayed_writeback_bundle(&projected_bundle, &changed_projected_bundle)
        .expect_err("cross-family replay/loop isolation same-family changed-causality drift must fail closed");
    let replay_validation_error = rebuilt_runtime
        .validate_replayed_writeback_bundle(&projected_bundle, &aspect_bundle)
        .expect_err(
            "cross-family replay/loop isolation cross-family replay validation must fail closed",
        );
    let family_replay_records = rebuilt_runtime.diagnostics().writeback_replay_records();
    let same_family_drift_replay_record = find_replay_record(
        &family_replay_records,
        projected_bundle.digest(),
        changed_projected_bundle.digest(),
    )
    .ok_or_else(|| {
        BridgeHarnessError::new(
            "cross-family replay/loop isolation same-family drift replay record missing",
        )
    })?;
    let cross_family_replay_record = find_replay_record(
        &family_replay_records,
        projected_bundle.digest(),
        aspect_bundle.digest(),
    )
    .ok_or_else(|| {
        BridgeHarnessError::new(
            "cross-family replay/loop isolation cross-family replay record missing",
        )
    })?;
    let counters = aggregate_runtime_writeback_counters(&[runtime_bridge, &rebuilt_runtime]);
    let counter_snapshot = snapshot_from_counters(&counters);
    let family_extension_digest = digest_string(
        "bridge-writeback-family-replay-loop-isolation",
        &format!(
            "projected={}|aspect={}|replay={:?}|loop={:?}",
            projected_bundle.digest(),
            aspect_bundle.digest(),
            replay_validation_error.kind(),
            cross_family_loop_prevention.disposition()
        ),
    )
    .to_string();
    let replay_loop_matrix = WritebackReplayLoopIsolationMatrix::from_replay_loop_evidence(
        WritebackReplayLoopIsolationMatrixEvidence {
            projected_effect: &projected_effect,
            aspect_effect: &aspect_effect,
            projected_idempotence: &projected_idempotence,
            aspect_idempotence: &aspect_idempotence,
            projected_bundle: &projected_bundle,
            aspect_bundle: &aspect_bundle,
            cross_family_replay_error: &replay_validation_error,
            cross_family_replay_record: &cross_family_replay_record,
            rebuilt_projected_effect: &rebuilt_projected_effect,
            rebuilt_projected_bundle: &rebuilt_projected_bundle,
            rebuilt_execution_record: &rebuilt_execution_record,
            changed_projected_bundle: &changed_projected_bundle,
            same_family_drift_error: &same_family_drift_error,
            same_family_drift_replay_record: &same_family_drift_replay_record,
            projected_feedback_context: &projected_feedback_context,
            cross_family_loop_prevention: &cross_family_loop_prevention,
        },
    );
    Ok(
        WritebackHarnessExecution::CrossFamilyReplayLoopIsolationCertification {
            family_extension_digest,
            replay_loop_matrix,
            counter_snapshot,
        },
    )
}
