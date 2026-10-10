use super::*;

pub(super) fn execute_duplicate_certification(
    _runtime: &crate::harness::adapter::BridgeHarnessSession,
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WritebackHarnessExecution, BridgeHarnessError> {
    let lowered_policy_bundle = lowered_policy(runtime_bridge)?;
    let declaration = crate::facade::BridgeWritebackDeclaration::writeback_capable(
        crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
            "harness:writeback-duplicate",
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
                "writeback duplicate certification contract admission failed: {error}"
            ))
        })?;

    let commit_identity = fixture
        .committed_patches()
        .first()
        .map(|patch| patch.commit_identity().clone())
        .ok_or_else(|| {
            BridgeHarnessError::new(
                "writeback duplicate certification fixture requires one committed patch",
            )
        })?;
    let route_identity =
        route_identity_for_commit(runtime_bridge, commit_identity.clone(), resource_request)?;
    let causality = writeback_causality_basis(
        "harness:writeback-causality",
        commit_identity.as_str(),
        route_identity.as_str(),
        "duplicate",
        fixture
            .snapshots()
            .first()
            .map(|snapshot| snapshot.identity().as_str())
            .unwrap_or("duplicate")
            .to_string(),
    );
    let effect = runtime_bridge.lower_writeback_effect(
        &contract,
        &causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-effect",
        ),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            contract.digest().to_owned(),
        ),
    );
    let first_idempotence = runtime_bridge.classify_writeback_idempotence(
        &effect,
        &lowered_policy_bundle,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(&effect),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-idempotence:first",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let repeated_idempotence = runtime_bridge.classify_writeback_idempotence(
        &effect,
        &lowered_policy_bundle,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(&effect),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-idempotence:repeat",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let (first_outcome, first_receipt) = runtime_bridge
        .execute_writeback_authority(&contract, &effect, &first_idempotence)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback duplicate certification first authority execution failed: {error}"
            ))
        })?;
    let (repeated_outcome, repeated_receipt) = runtime_bridge
        .execute_writeback_authority(&contract, &effect, &repeated_idempotence)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback duplicate certification repeated authority execution failed: {error}"
            ))
        })?;
    let first_bundle = runtime_bridge.replay_writeback_bundle(
        &contract,
        &effect,
        &first_idempotence,
        &first_outcome,
    );
    let repeated_bundle = runtime_bridge.replay_writeback_bundle(
        &contract,
        &effect,
        &repeated_idempotence,
        &repeated_outcome,
    );
    let replay_bundle = runtime_bridge.replay_writeback_bundle(
        &contract,
        &effect,
        &repeated_idempotence,
        &repeated_outcome,
    );
    let commit_count = match (
        first_receipt.outcome_class(),
        repeated_receipt.outcome_class(),
    ) {
        (
            crate::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit,
            crate::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit,
        ) => 2,
        (crate::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit, _)
        | (_, crate::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit) => 1,
        _ => 0,
    };
    let noop_count = match (
        first_receipt.outcome_class(),
        repeated_receipt.outcome_class(),
    ) {
        (
            crate::facade::BridgeWritebackOutcomeClass::CanonicalNoop,
            crate::facade::BridgeWritebackOutcomeClass::CanonicalNoop,
        ) => 2,
        (crate::facade::BridgeWritebackOutcomeClass::CanonicalNoop, _)
        | (_, crate::facade::BridgeWritebackOutcomeClass::CanonicalNoop) => 1,
        _ => 0,
    };
    let counters = aggregate_runtime_writeback_counters(&[runtime_bridge]);
    let counter_snapshot = snapshot_from_counters(&counters);
    let first_loop_prevention =
        runtime_bridge.classify_writeback_loop_prevention(&effect, &first_idempotence, None);
    let repeated_loop_prevention =
        runtime_bridge.classify_writeback_loop_prevention(&effect, &repeated_idempotence, None);
    let first_strategy_coherence = runtime_bridge.classify_writeback_strategy_coherence(
        &contract,
        &effect,
        &first_idempotence,
    );
    let repeated_strategy_coherence = runtime_bridge.classify_writeback_strategy_coherence(
        &contract,
        &effect,
        &repeated_idempotence,
    );
    let first_feedback_provenance = runtime_bridge.derive_writeback_feedback_provenance(&effect);
    let first_candidate = runtime_bridge
        .validate_writeback_candidate(
            &contract,
            &effect,
            &first_idempotence,
            &first_loop_prevention,
            &first_strategy_coherence,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback duplicate certification first candidate validation failed: {error}"
            ))
        })?;
    let repeated_candidate = runtime_bridge
        .validate_writeback_candidate(
            &contract,
            &effect,
            &repeated_idempotence,
            &repeated_loop_prevention,
            &repeated_strategy_coherence,
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback duplicate certification repeated candidate validation failed: {error}"
            ))
        })?;
    let mapped_input = runtime_bridge
        .diagnostics()
        .writeback_mapped_family_input_for_digest(effect.mapped_input_digest())
        .expect("writeback harness should retain mapped-family input for duplicate certification");
    let mapper_witness = crate::facade::BridgeWritebackMapperWitness::issue(&mapped_input);
    let first_authority_request = crate::adapter::TruthWritebackRequest::from_evidence(
        crate::adapter::TruthWritebackRequestEvidence {
            contract: &contract,
            candidate: &first_candidate,
            effect: &effect,
            mapper_witness: &mapper_witness,
            feedback_provenance: &first_feedback_provenance,
            loop_prevention: &first_loop_prevention,
            strategy_coherence: &first_strategy_coherence,
            idempotence: &first_idempotence,
        },
    );
    let repeated_authority_request = crate::adapter::TruthWritebackRequest::from_evidence(
        crate::adapter::TruthWritebackRequestEvidence {
            contract: &contract,
            candidate: &repeated_candidate,
            effect: &effect,
            mapper_witness: &mapper_witness,
            feedback_provenance: &first_feedback_provenance,
            loop_prevention: &repeated_loop_prevention,
            strategy_coherence: &repeated_strategy_coherence,
            idempotence: &repeated_idempotence,
        },
    );

    Ok(WritebackHarnessExecution::DuplicateCertification {
        first_bundle_digest: first_bundle.digest().to_string(),
        repeated_bundle_digest: repeated_bundle.digest().to_string(),
        replay_bundle_digest: replay_bundle.digest().to_string(),
        duplicate_authority_matrix: WritebackDuplicateAuthorityMatrix::from_duplicate_attempts(
            WritebackDuplicateAuthorityMatrixEvidence {
                contract: &contract,
                effect: &effect,
                causality: &causality,
                replay_bundle: &replay_bundle,
                first_bundle: &first_bundle,
                repeated_bundle: &repeated_bundle,
                first_idempotence: &first_idempotence,
                repeated_idempotence: &repeated_idempotence,
                first_loop_prevention: &first_loop_prevention,
                repeated_loop_prevention: &repeated_loop_prevention,
                first_strategy_coherence: &first_strategy_coherence,
                repeated_strategy_coherence: &repeated_strategy_coherence,
                first_candidate: &first_candidate,
                repeated_candidate: &repeated_candidate,
                first_authority_request: &first_authority_request,
                repeated_authority_request: &repeated_authority_request,
                first_receipt: &first_receipt,
                repeated_receipt: &repeated_receipt,
                first_outcome: &first_outcome,
                repeated_outcome: &repeated_outcome,
                commit_count,
                noop_count,
            },
        ),
        counter_snapshot,
    })
}
