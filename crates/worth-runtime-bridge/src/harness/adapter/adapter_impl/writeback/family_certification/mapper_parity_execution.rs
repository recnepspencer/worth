use super::*;

pub(in crate::harness::adapter::adapter_impl::writeback) fn execute_host_mapper_parity_certification(
    runtime: &crate::harness::adapter::BridgeHarnessSession,
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WritebackHarnessExecution, BridgeHarnessError> {
    let _ = runtime;
    let lowered_policy_bundle = lowered_policy(runtime_bridge)?;
    let causality = writeback_causality_basis(
        "harness:writeback-family-mapper-parity-causality",
        "family-mapper-parity",
        route_digest_for_first_patch(runtime_bridge, fixture, resource_request)?,
        "family-mapper-parity",
        "family-mapper-parity",
    );
    let projected_contract = runtime_bridge
        .admit_writeback_declaration(
            crate::facade::BridgeWritebackDeclaration::writeback_capable(
                crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "harness:writeback-family-mapper-parity:projected",
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
                "host mapper parity projected family admission failed: {error}"
            ))
        })?;
    let aspect_contract = runtime_bridge
        .admit_writeback_declaration(
            crate::facade::BridgeWritebackDeclaration::writeback_capable(
                crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "harness:writeback-family-mapper-parity:aspect",
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
                "host mapper parity aspect family admission failed: {error}"
            ))
        })?;
    let projected_effect = runtime_bridge.lower_writeback_effect(
        &projected_contract,
        &causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-family-mapper-parity:effect:projected",
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
            "harness:writeback-family-mapper-parity:effect:aspect",
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
            "harness:writeback-family-mapper-parity:idempotence:projected",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let aspect_idempotence = runtime_bridge.classify_writeback_idempotence(
        &aspect_effect,
        &lowered_policy_bundle,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(&aspect_effect),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-family-mapper-parity:idempotence:aspect",
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
                "host mapper parity projected family execution failed: {error}"
            ))
        })?;
    let (aspect_outcome, _) = runtime_bridge
        .execute_writeback_authority(&aspect_contract, &aspect_effect, &aspect_idempotence)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "host mapper parity aspect family execution failed: {error}"
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
    let family_execution_records = runtime_bridge.diagnostics().writeback_execution_records();
    let projected_execution_record =
        find_execution_record_for_replay(&family_execution_records, projected_bundle.digest())
            .ok_or_else(|| {
                BridgeHarnessError::new(
            "host mapper parity projected execution record missing from retained diagnostics",
        )
            })?;
    let aspect_execution_record =
        find_execution_record_for_replay(&family_execution_records, aspect_bundle.digest())
            .ok_or_else(|| {
                BridgeHarnessError::new(
                    "host mapper parity aspect execution record missing from retained diagnostics",
                )
            })?;
    let projected_admission_record = runtime_bridge
        .diagnostics()
        .writeback_admission_record_for_contract_digest(projected_contract.digest())
        .ok_or_else(|| {
            BridgeHarnessError::new(
                "host mapper parity projected admission record missing from retained diagnostics",
            )
        })?;
    let aspect_admission_record = runtime_bridge
        .diagnostics()
        .writeback_admission_record_for_contract_digest(aspect_contract.digest())
        .ok_or_else(|| {
            BridgeHarnessError::new(
                "host mapper parity aspect admission record missing from retained diagnostics",
            )
        })?;
    let shadow_protocol_error = runtime_bridge
        .validate_writeback_declaration(crate::facade::BridgeWritebackDeclaration::new(
            crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                "harness:writeback-family-mapper-parity:shadow-protocol",
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
        .expect_err("host mapper parity shadow protocol mismatch must fail closed");
    let counters = aggregate_runtime_writeback_counters(&[runtime_bridge]);
    let counter_snapshot = snapshot_from_counters(&counters);
    let family_extension_digest = digest_string(
        "bridge-writeback-family-mapper-parity",
        &format!(
            "projected={}|aspect={}|shadow={:?}",
            projected_bundle.digest(),
            aspect_bundle.digest(),
            shadow_protocol_error.kind()
        ),
    )
    .to_string();
    let mapper_parity_matrix = WritebackMapperParityMatrix::from_mapper_parity_evidence(
        WritebackMapperParityMatrixEvidence {
            projected_effect: &projected_effect,
            aspect_effect: &aspect_effect,
            projected_replay_bundle: &projected_bundle,
            aspect_replay_bundle: &aspect_bundle,
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
                .writeback_mapped_family_input_for_digest(projected_effect.mapped_input_digest())
                .is_some(),
            aspect_mapped_input_retained: runtime_bridge
                .diagnostics()
                .writeback_mapped_family_input_for_digest(aspect_effect.mapped_input_digest())
                .is_some(),
            projected_execution_record: &projected_execution_record,
            aspect_execution_record: &aspect_execution_record,
            projected_admission_record_digest: projected_admission_record.digest(),
            aspect_admission_record_digest: aspect_admission_record.digest(),
            shadow_protocol_error: &shadow_protocol_error,
        },
    );
    Ok(WritebackHarnessExecution::HostMapperParityCertification {
        family_extension_digest,
        mapper_parity_matrix,
        counter_snapshot,
    })
}
