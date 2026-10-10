use super::*;

pub(in crate::harness::adapter::adapter_impl::writeback) fn execute_multi_family_admission_boundary_certification(
    runtime: &crate::harness::adapter::BridgeHarnessSession,
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WritebackHarnessExecution, BridgeHarnessError> {
    let _ = runtime;
    let lowered_policy_bundle = lowered_policy(runtime_bridge)?;
    let causality = writeback_causality_basis(
        "harness:writeback-family-admission-boundary-causality",
        "family-admission-boundary",
        route_digest_for_first_patch(runtime_bridge, fixture, resource_request)?,
        "family-admission-boundary",
        "family-admission-boundary",
    );
    let projected_contract = runtime_bridge
        .admit_writeback_declaration(
            crate::facade::BridgeWritebackDeclaration::writeback_capable(
                crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "harness:writeback-family-admission-boundary:projected",
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
                "multi-family admission boundary projected family admission failed: {error}"
            ))
        })?;
    let aspect_contract = runtime_bridge
        .admit_writeback_declaration(
            crate::facade::BridgeWritebackDeclaration::writeback_capable(
                crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "harness:writeback-family-admission-boundary:aspect",
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
                "multi-family admission boundary aspect family admission failed: {error}"
            ))
        })?;
    let projected_effect = runtime_bridge.lower_writeback_effect(
        &projected_contract,
        &causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-family-admission-boundary:effect:projected",
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
            "harness:writeback-family-admission-boundary:effect:aspect",
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
            "harness:writeback-family-admission-boundary:idempotence:projected",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let aspect_idempotence = runtime_bridge.classify_writeback_idempotence(
        &aspect_effect,
        &lowered_policy_bundle,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(&aspect_effect),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-family-admission-boundary:idempotence:aspect",
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
                "multi-family admission boundary projected family execution failed: {error}"
            ))
        })?;
    let (aspect_outcome, _) = runtime_bridge
        .execute_writeback_authority(&aspect_contract, &aspect_effect, &aspect_idempotence)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "multi-family admission boundary aspect family execution failed: {error}"
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
    let projected_admission_record = runtime_bridge
        .diagnostics()
        .writeback_admission_record_for_contract_digest(projected_contract.digest())
        .ok_or_else(|| {
            BridgeHarnessError::new(
                "multi-family admission boundary projected admission record missing",
            )
        })?;
    let aspect_admission_record = runtime_bridge
        .diagnostics()
        .writeback_admission_record_for_contract_digest(aspect_contract.digest())
        .ok_or_else(|| {
            BridgeHarnessError::new(
                "multi-family admission boundary aspect admission record missing",
            )
        })?;
    let shadow_protocol_error = runtime_bridge
        .validate_writeback_declaration(crate::facade::BridgeWritebackDeclaration::new(
            crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                "harness:writeback-family-admission-boundary:shadow-protocol",
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
        .expect_err("multi-family admission boundary shadow protocol mismatch must fail closed");
    let counters = aggregate_runtime_writeback_counters(&[runtime_bridge]);
    let counter_snapshot = snapshot_from_counters(&counters);
    let family_extension_digest = digest_string(
        "bridge-writeback-family-admission-boundary",
        &format!(
            "projected={}|aspect={}|shadow={:?}",
            projected_bundle.digest(),
            aspect_bundle.digest(),
            shadow_protocol_error.kind()
        ),
    )
    .to_string();
    let admission_boundary_matrix =
        WritebackAdmissionBoundaryMatrix::from_admission_boundary_evidence(
            WritebackAdmissionBoundaryMatrixEvidence {
                projected_contract: &projected_contract,
                aspect_contract: &aspect_contract,
                projected_admission_record_digest: projected_admission_record.digest(),
                aspect_admission_record_digest: aspect_admission_record.digest(),
                projected_effect: &projected_effect,
                aspect_effect: &aspect_effect,
                projected_idempotence: &projected_idempotence,
                aspect_idempotence: &aspect_idempotence,
                projected_bundle: &projected_bundle,
                aspect_bundle: &aspect_bundle,
                projected_authority_outcome: &projected_outcome,
                aspect_authority_outcome: &aspect_outcome,
                shadow_protocol_error: &shadow_protocol_error,
            },
        );
    Ok(
        WritebackHarnessExecution::MultiFamilyAdmissionBoundaryCertification {
            family_extension_digest,
            admission_boundary_matrix,
            counter_snapshot,
        },
    )
}
