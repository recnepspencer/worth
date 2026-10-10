use super::*;

pub(in crate::harness::adapter::adapter_impl::writeback) fn execute_extensible_family_certification(
    runtime: &crate::harness::adapter::BridgeHarnessSession,
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WritebackHarnessExecution, BridgeHarnessError> {
    let lowered_policy_bundle = lowered_policy(runtime_bridge)?;
    let causality = writeback_causality_basis(
        "harness:writeback-family-extension-causality",
        "family-extension",
        route_digest_for_first_patch(runtime_bridge, fixture, resource_request)?,
        "family-extension",
        "family-extension",
    );
    let projected_contract = runtime_bridge
        .admit_writeback_declaration(
            crate::facade::BridgeWritebackDeclaration::writeback_capable(
                crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "harness:writeback-family-extension:projected",
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
            BridgeHarnessError::new(format!("projected family admission failed: {error}"))
        })?;
    let aspect_contract = runtime_bridge
        .admit_writeback_declaration(
            crate::facade::BridgeWritebackDeclaration::writeback_capable(
                crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "harness:writeback-family-extension:aspect",
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
            BridgeHarnessError::new(format!("aspect family admission failed: {error}"))
        })?;
    let projected_effect = runtime_bridge.lower_writeback_effect(
        &projected_contract,
        &causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-family-extension:effect:projected",
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
            "harness:writeback-family-extension:effect:aspect",
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
            "harness:writeback-family-extension:idempotence:projected",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let aspect_idempotence = runtime_bridge.classify_writeback_idempotence(
        &aspect_effect,
        &lowered_policy_bundle,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(&aspect_effect),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-family-extension:idempotence:aspect",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let (projected_outcome, projected_receipt) = runtime_bridge
        .execute_writeback_authority(
            &projected_contract,
            &projected_effect,
            &projected_idempotence,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "projected family authority execution failed: {error}"
            ))
        })?;
    let (aspect_outcome, aspect_receipt) = runtime_bridge
        .execute_writeback_authority(&aspect_contract, &aspect_effect, &aspect_idempotence)
        .map_err(|error| {
            BridgeHarnessError::new(format!("aspect family authority execution failed: {error}"))
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
    let projected_admission_record = runtime_bridge
        .diagnostics()
        .writeback_admission_record_for_contract_digest(projected_contract.digest())
        .ok_or_else(|| {
            BridgeHarnessError::new(
                "projected family admission record missing from retained diagnostics",
            )
        })?;
    let aspect_admission_record = runtime_bridge
        .diagnostics()
        .writeback_admission_record_for_contract_digest(aspect_contract.digest())
        .ok_or_else(|| {
            BridgeHarnessError::new(
                "aspect family admission record missing from retained diagnostics",
            )
        })?;
    let family_execution_records = runtime_bridge.diagnostics().writeback_execution_records();
    let projected_execution_record =
        find_execution_record_for_replay(&family_execution_records, projected_bundle.digest())
            .ok_or_else(|| {
                BridgeHarnessError::new(
                    "projected family execution record missing from retained diagnostics",
                )
            })?;
    let aspect_execution_record =
        find_execution_record_for_replay(&family_execution_records, aspect_bundle.digest())
            .ok_or_else(|| {
                BridgeHarnessError::new(
                    "aspect family execution record missing from retained diagnostics",
                )
            })?;

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
                    "harness:writeback-family-extension:projected",
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
                "rebuilt projected family admission failed during extensible certification: {error}"
            ))
        })?;
    let rebuilt_projected_effect = rebuilt_runtime.lower_writeback_effect(
        &rebuilt_projected_contract,
        &causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-family-extension:effect:projected",
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
            "harness:writeback-family-extension:idempotence:projected",
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
                "rebuilt projected family execution failed during extensible certification: {error}"
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
                "same-family rebuilt replay validation unexpectedly failed: {error}"
            ))
        })?;
    let rebuilt_execution_record = rebuilt_runtime
        .diagnostics()
        .last_writeback_execution_record()
        .ok_or_else(|| {
            BridgeHarnessError::new(
                "rebuilt projected execution record missing from retained diagnostics",
            )
        })?;
    let changed_causality = writeback_causality_basis(
        "harness:writeback-family-extension-causality:changed",
        causality.digest(),
        causality.digest(),
        "family-extension",
        "family-extension",
    );
    let changed_projected_effect = rebuilt_runtime.lower_writeback_effect(
        &rebuilt_projected_contract,
        &changed_causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-family-extension:effect:projected:changed",
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
            "harness:writeback-family-extension:idempotence:projected:changed",
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
                "changed-causality projected execution failed during extensible certification: {error}"
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
        .expect_err("same-family changed-causality replay validation must fail closed");

    let replay_validation_error = rebuilt_runtime
        .validate_replayed_writeback_bundle(&projected_bundle, &aspect_bundle)
        .expect_err("cross-family replay validation must fail closed");
    let family_replay_records = rebuilt_runtime.diagnostics().writeback_replay_records();
    let same_family_drift_replay_record = find_replay_record(
        &family_replay_records,
        projected_bundle.digest(),
        changed_projected_bundle.digest(),
    )
    .ok_or_else(|| {
        BridgeHarnessError::new(
            "same-family changed-causality replay record missing from retained diagnostics",
        )
    })?;
    let cross_family_replay_record = find_replay_record(
        &family_replay_records,
        projected_bundle.digest(),
        aspect_bundle.digest(),
    )
    .ok_or_else(|| {
        BridgeHarnessError::new(
            "cross-family replay record missing from retained diagnostics after mismatch validation",
        )
    })?;

    let shadow_protocol_error = runtime_bridge
        .validate_writeback_declaration(crate::facade::BridgeWritebackDeclaration::new(
            crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                "harness:writeback-family-extension:shadow-protocol",
            ),
            crate::facade::BridgeRequestKind::Authoritative,
            crate::facade::BridgeWritebackRequestMode::WritebackCapable,
            Some(crate::facade::BridgeWritebackFamilyKind::ProjectedStateDiff),
            crate::facade::BridgeWritebackEffectClass::AspectReconciliation,
            Some(crate::facade::BridgeWritebackStrategyClass::AspectReconciliationCommit),
            Some(
                crate::facade::BridgeWritebackStrategyDescriptorBasis::for_writeback_contract(
                    crate::facade::BridgeWritebackFamilyKind::ProjectedStateDiff,
                    crate::facade::BridgeWritebackEffectClass::AspectReconciliation,
                    crate::facade::BridgeWritebackStrategyClass::AspectReconciliationCommit,
                    crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
                ),
            ),
            crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
        ))
        .expect_err("shadow protocol family/effect mismatch must fail closed");
    let counters = aggregate_runtime_writeback_counters(&[runtime_bridge, &rebuilt_runtime]);
    let counter_snapshot = snapshot_from_counters(&counters);

    Ok(WritebackHarnessExecution::ExtensibleFamilyCertification {
        family_extension_digest: digest_string(
            "bridge-writeback-family-extension",
            &format!(
                "projected={}|aspect={}|shadow={:?}",
                projected_bundle.digest(),
                aspect_bundle.digest(),
                shadow_protocol_error.kind()
            ),
        )
        .to_string(),
        family_extension_matrix: WritebackFamilyExtensionMatrix::from_family_extension_evidence(
            WritebackFamilyExtensionMatrixEvidence {
                projected_contract: &projected_contract,
                aspect_contract: &aspect_contract,
                projected_admission_record: &projected_admission_record,
                aspect_admission_record: &aspect_admission_record,
                projected_effect: &projected_effect,
                aspect_effect: &aspect_effect,
                projected_idempotence: &projected_idempotence,
                aspect_idempotence: &aspect_idempotence,
                projected_bundle: &projected_bundle,
                aspect_bundle: &aspect_bundle,
                projected_receipt: &projected_receipt,
                aspect_receipt: &aspect_receipt,
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
                projected_mapper_envelope_retained: runtime_bridge
                    .diagnostics()
                    .writeback_mapper_envelope_for_digest(projected_effect.mapper_envelope_digest())
                    .is_some(),
                aspect_mapper_envelope_retained: runtime_bridge
                    .diagnostics()
                    .writeback_mapper_envelope_for_digest(aspect_effect.mapper_envelope_digest())
                    .is_some(),
                projected_mapped_input_retained: runtime_bridge
                    .diagnostics()
                    .writeback_mapped_family_input_for_digest(
                        projected_effect.mapped_input_digest(),
                    )
                    .is_some(),
                aspect_mapped_input_retained: runtime_bridge
                    .diagnostics()
                    .writeback_mapped_family_input_for_digest(aspect_effect.mapped_input_digest())
                    .is_some(),
                projected_execution_record: &projected_execution_record,
                aspect_execution_record: &aspect_execution_record,
                shadow_protocol_error: &shadow_protocol_error,
            },
        ),
        counter_snapshot,
    })
}
