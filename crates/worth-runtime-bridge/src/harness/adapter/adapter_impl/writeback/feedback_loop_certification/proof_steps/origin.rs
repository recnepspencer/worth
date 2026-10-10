use super::super::*;

pub(in crate::harness::adapter::adapter_impl::writeback::feedback_loop_certification) struct FeedbackOriginProof
{
    pub lowered_policy_bundle: crate::facade::LoweredBridgeExecutionPolicy,
    pub contract: crate::facade::AdmittedBridgeWritebackContract,
    pub original_commit: crate::facade::BridgeCommittedPatchEnvelope,
    pub initial_route_digest: String,
    pub original_causality: crate::facade::BridgeWritebackNativeCausalityInputs,
    pub effect: crate::writeback::BridgeDerivedWritebackEffect,
    pub feedback_provenance: crate::facade::BridgeWritebackFeedbackProvenance,
    pub feedback_context: crate::facade::BridgeWritebackFeedbackContext,
    pub initial_idempotence: crate::facade::BridgeWritebackIdempotenceBasis,
    pub initial_outcome: crate::facade::BridgeWritebackAuthorityOutcome,
}

pub(in crate::harness::adapter::adapter_impl::writeback::feedback_loop_certification) fn establish_feedback_origin_proof(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<FeedbackOriginProof, BridgeHarnessError> {
    let lowered_policy_bundle = lowered_policy(runtime_bridge)?;
    let declaration = crate::facade::BridgeWritebackDeclaration::writeback_capable(
        crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
            "harness:writeback-feedback",
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
                "writeback feedback certification contract admission failed: {error}"
            ))
        })?;

    let original_commit = fixture
        .committed_patches()
        .first()
        .cloned()
        .ok_or_else(|| {
            BridgeHarnessError::new("writeback feedback fixture requires one committed patch")
        })?;
    let initial_route_digest =
        route_digest_for_first_patch(runtime_bridge, fixture, resource_request)?;
    let original_causality = writeback_causality_basis(
        "harness:writeback-feedback-causality",
        original_commit.commit_identity().as_str(),
        initial_route_digest.clone(),
        "feedback",
        original_commit.snapshot_identity().as_str(),
    );
    let effect = runtime_bridge.lower_writeback_effect(
        &contract,
        &original_causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-feedback-effect",
        ),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            contract.digest().to_owned(),
        ),
    );
    let feedback_provenance = runtime_bridge.derive_writeback_feedback_provenance(&effect);
    let feedback_context =
        crate::facade::BridgeWritebackFeedbackContext::from_provenance(&feedback_provenance);
    let initial_idempotence = runtime_bridge.classify_writeback_idempotence(
        &effect,
        &lowered_policy_bundle,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(&effect),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-feedback-idempotence:first",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let (initial_outcome, _initial_receipt) = runtime_bridge
        .execute_writeback_authority(&contract, &effect, &initial_idempotence)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback feedback certification first authority execution failed: {error}"
            ))
        })?;

    Ok(FeedbackOriginProof {
        lowered_policy_bundle,
        contract,
        original_commit,
        initial_route_digest,
        original_causality,
        effect,
        feedback_provenance,
        feedback_context,
        initial_idempotence,
        initial_outcome,
    })
}
