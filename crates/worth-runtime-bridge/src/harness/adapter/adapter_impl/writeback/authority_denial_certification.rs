use super::*;
use crate::harness::adapter::adapter_impl::writeback_certification::AuthorityDenialBoundaryClass;

pub(super) fn execute_authority_denial_certification(
    runtime: &crate::harness::adapter::BridgeHarnessSession,
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<WritebackHarnessExecution, BridgeHarnessError> {
    let declaration = crate::facade::BridgeWritebackDeclaration::writeback_capable(
        crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
            "harness:writeback-authority-denial-preview",
        ),
        crate::facade::BridgeRequestKind::Preview,
        crate::facade::BridgeWritebackFamilyKind::ProjectedStateDiff,
        crate::facade::BridgeWritebackEffectClass::ProjectedStateDiff,
        crate::facade::BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let error = runtime_bridge
        .validate_writeback_declaration(declaration)
        .expect_err("preview writeback authority denial must fail closed");
    let error_message = error.to_string();
    let failure_digest = digest_string(
        "bridge-writeback-harness-failure",
        &format!("{:?}|{}", error.kind(), error_message),
    )
    .to_string();
    let unbound_runtime = build_writeback_runtime(runtime, fixture, false)?;
    let lowered_policy_bundle = lowered_policy(&unbound_runtime)?;
    let authority_declaration = crate::facade::BridgeWritebackDeclaration::writeback_capable(
        crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
            "harness:writeback-authority-denial:unbound-authority",
        ),
        crate::facade::BridgeRequestKind::Authoritative,
        crate::facade::BridgeWritebackFamilyKind::ProjectedStateDiff,
        crate::facade::BridgeWritebackEffectClass::ProjectedStateDiff,
        crate::facade::BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let authority_contract = unbound_runtime
        .admit_writeback_declaration(authority_declaration, &lowered_policy_bundle)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback authority-denial certification failed to admit unbound-authority contract: {error}"
            ))
        })?;
    let authority_causality = authority_denial_causality(
        runtime_bridge,
        fixture,
        "harness:writeback-authority-denial:causality",
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        "unbound-authority",
        resource_request,
    )?;
    let authority_effect = unbound_runtime.lower_writeback_effect(
        &authority_contract,
        &authority_causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-authority-denial:effect",
        ),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            authority_contract.digest().to_owned(),
        ),
    );
    let authority_idempotence = unbound_runtime.classify_writeback_idempotence(
        &authority_effect,
        &lowered_policy_bundle,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(&authority_effect),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-authority-denial:idempotence",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let authority_strategy_coherence = unbound_runtime.classify_writeback_strategy_coherence(
        &authority_contract,
        &authority_effect,
        &authority_idempotence,
    );
    let authority_error = unbound_runtime
        .execute_writeback_authority(
            &authority_contract,
            &authority_effect,
            &authority_idempotence,
        )
        .expect_err("unbound writeback authority execution must fail closed");
    let authority_failure_digest = writeback_harness_error_digest(
        WritebackHarnessErrorDigestDomain::AuthorityDenial,
        authority_error.kind(),
        &authority_error,
    );
    let merge_rejecting_authority = RejectingTruthWritebackAuthority::new(
        crate::facade::BridgeWritebackFailureClass::MergeAuthorityRejected,
    );
    let merge_rejecting_runtime = build_writeback_runtime_with_custom_authority(
        runtime,
        fixture,
        merge_rejecting_authority.clone(),
    )?;
    let merge_lowered_policy = lowered_policy(&merge_rejecting_runtime)?;
    let merge_declaration = crate::facade::BridgeWritebackDeclaration::writeback_capable(
        crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
            "harness:writeback-authority-denial:merge-rejected",
        ),
        crate::facade::BridgeRequestKind::Authoritative,
        crate::facade::BridgeWritebackFamilyKind::ProjectedStateDiff,
        crate::facade::BridgeWritebackEffectClass::ProjectedStateDiff,
        crate::facade::BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let merge_contract = merge_rejecting_runtime
        .admit_writeback_declaration(merge_declaration, &merge_lowered_policy)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback authority-denial certification failed to admit merge-rejected contract: {error}"
            ))
        })?;
    let merge_causality = authority_denial_causality(
        &merge_rejecting_runtime,
        fixture,
        "harness:writeback-authority-denial:merge-rejected:causality",
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        "merge-rejected",
        resource_request,
    )?;
    let merge_effect = merge_rejecting_runtime.lower_writeback_effect(
        &merge_contract,
        &merge_causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-authority-denial:merge-rejected:effect",
        ),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            merge_contract.digest().to_owned(),
        ),
    );
    let merge_idempotence = merge_rejecting_runtime.classify_writeback_idempotence(
        &merge_effect,
        &merge_lowered_policy,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(&merge_effect),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-authority-denial:merge-rejected:idempotence",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let merge_strategy_coherence = merge_rejecting_runtime.classify_writeback_strategy_coherence(
        &merge_contract,
        &merge_effect,
        &merge_idempotence,
    );
    let merge_error = merge_rejecting_runtime
        .execute_writeback_authority(&merge_contract, &merge_effect, &merge_idempotence)
        .expect_err("merge-authority rejection must fail closed");
    let merge_failure_digest = writeback_harness_error_digest(
        WritebackHarnessErrorDigestDomain::MergeAuthorityDenial,
        merge_error.kind(),
        &merge_error,
    );
    let merge_authority_request = merge_rejecting_authority.last_request();
    let merge_authority_receipt = merge_rejecting_authority.last_receipt();
    let unsafe_feedback_runtime = build_writeback_runtime(runtime, fixture, true)?;
    let unsafe_feedback_policy = lowered_policy(&unsafe_feedback_runtime)?;
    let unsafe_feedback_declaration = crate::facade::BridgeWritebackDeclaration::writeback_capable(
        crate::facade::BridgeWritebackDeclarationIdentity::admit_bridge_owned(
            "harness:writeback-authority-denial:unsafe-feedback",
        ),
        crate::facade::BridgeRequestKind::Authoritative,
        crate::facade::BridgeWritebackFamilyKind::ProjectedStateDiff,
        crate::facade::BridgeWritebackEffectClass::ProjectedStateDiff,
        crate::facade::BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let unsafe_feedback_contract = unsafe_feedback_runtime
        .admit_writeback_declaration(unsafe_feedback_declaration, &unsafe_feedback_policy)
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "writeback authority-denial certification failed to admit unsafe-feedback contract: {error}"
            ))
        })?;
    let unsafe_feedback_causality = authority_denial_causality(
        &unsafe_feedback_runtime,
        fixture,
        "harness:writeback-authority-denial:unsafe-feedback:causality",
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        "unsafe-feedback",
        resource_request,
    )?;
    let unsafe_feedback_effect = unsafe_feedback_runtime.lower_writeback_effect(
        &unsafe_feedback_contract,
        &unsafe_feedback_causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-authority-denial:unsafe-feedback:effect",
        ),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            unsafe_feedback_contract.digest().to_owned(),
        ),
    );
    let unsafe_feedback_idempotence = unsafe_feedback_runtime.classify_writeback_idempotence(
        &unsafe_feedback_effect,
        &unsafe_feedback_policy,
        &crate::facade::BridgeWritebackAuthoritativeStateBasis::from_effect(
            &unsafe_feedback_effect,
        ),
        crate::facade::BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "harness:writeback-authority-denial:unsafe-feedback:idempotence",
        ),
        crate::facade::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );
    let unsafe_feedback_strategy_coherence = unsafe_feedback_runtime
        .classify_writeback_strategy_coherence(
            &unsafe_feedback_contract,
            &unsafe_feedback_effect,
            &unsafe_feedback_idempotence,
        );
    let unsafe_feedback_drift_effect = unsafe_feedback_runtime.lower_writeback_effect(
        &unsafe_feedback_contract,
        &unsafe_feedback_causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-authority-denial:unsafe-feedback:effect:drift",
        ),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            unsafe_feedback_effect.digest().to_owned(),
        ),
    );
    let unsafe_feedback_drift_provenance =
        unsafe_feedback_runtime.derive_writeback_feedback_provenance(&unsafe_feedback_drift_effect);
    let unsafe_feedback_drift_context =
        crate::facade::BridgeWritebackFeedbackContext::from_provenance(
            &unsafe_feedback_drift_provenance,
        );
    let unsafe_feedback_loop_prevention = unsafe_feedback_runtime
        .classify_writeback_loop_prevention(
            &unsafe_feedback_effect,
            &unsafe_feedback_idempotence,
            Some(&unsafe_feedback_drift_context),
        );
    let unsafe_feedback_error = unsafe_feedback_runtime
        .execute_writeback_authority_with_feedback_context(
            &unsafe_feedback_contract,
            &unsafe_feedback_effect,
            &unsafe_feedback_idempotence,
            Some(&unsafe_feedback_drift_context),
        )
        .expect_err("drifted-effect feedback context must fail closed before authority execution");
    let unsafe_feedback_failure_digest = writeback_harness_error_digest(
        WritebackHarnessErrorDigestDomain::UnsafeFeedbackDenial,
        unsafe_feedback_error.kind(),
        &unsafe_feedback_error,
    );
    let contradictory_feedback_drift_effect = unsafe_feedback_runtime.lower_writeback_effect(
        &unsafe_feedback_contract,
        &unsafe_feedback_causality,
        crate::facade::BridgeWritebackEffectIdentity::admit_bridge_owned(
            "harness:writeback-authority-denial:unsafe-feedback:effect:contradictory",
        ),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            unsafe_feedback_idempotence.digest().to_owned(),
        ),
    );
    let contradictory_feedback_provenance = unsafe_feedback_runtime
        .derive_writeback_feedback_provenance(&contradictory_feedback_drift_effect);
    let contradictory_feedback_context =
        crate::facade::BridgeWritebackFeedbackContext::from_provenance(
            &contradictory_feedback_provenance,
        );
    let contradictory_feedback_loop_prevention = unsafe_feedback_runtime
        .classify_writeback_loop_prevention(
            &unsafe_feedback_effect,
            &unsafe_feedback_idempotence,
            Some(&contradictory_feedback_context),
        );
    let contradictory_feedback_error = unsafe_feedback_runtime
        .execute_writeback_authority_with_feedback_context(
            &unsafe_feedback_contract,
            &unsafe_feedback_effect,
            &unsafe_feedback_idempotence,
            Some(&contradictory_feedback_context),
        )
        .expect_err(
            "contradictory native feedback context must fail closed before authority execution",
        );
    let contradictory_feedback_failure_digest = writeback_harness_error_digest(
        WritebackHarnessErrorDigestDomain::ContradictoryFeedbackDenial,
        contradictory_feedback_error.kind(),
        &contradictory_feedback_error,
    );

    let counters = aggregate_runtime_writeback_counters(&[
        &unbound_runtime,
        &merge_rejecting_runtime,
        &unsafe_feedback_runtime,
    ]);
    let counter_snapshot = snapshot_from_counters(&counters);

    Ok(WritebackHarnessExecution::AuthorityDenialCertification {
        failure_digest,
        authority_denial: WritebackAuthorityDenialMatrix::from_authority_evidence(
            &error,
            error_message,
            &unsafe_feedback_loop_prevention,
            &contradictory_feedback_loop_prevention,
            AuthorityDenialBoundaryEvidence {
                validation_error: &error,
                unbound_authority: AuthorityDenialBoundaryFailureEvidence {
                    contract: Some(&authority_contract),
                    strategy_basis: authority_contract.validated_declaration().strategy_basis(),
                    strategy_coherence: Some(&authority_strategy_coherence),
                    authority_request: None,
                    authority_receipt: None,
                    denial_class: AuthorityDenialBoundaryClass::UnboundAuthority,
                    failure_kind: authority_error.kind(),
                    failure_digest: Some(&authority_failure_digest),
                    effect: Some(&authority_effect),
                    idempotence: Some(&authority_idempotence),
                    incoming_feedback_context: None,
                },
                merge_authority: AuthorityDenialBoundaryFailureEvidence {
                    contract: Some(&merge_contract),
                    strategy_basis: merge_contract.validated_declaration().strategy_basis(),
                    strategy_coherence: Some(&merge_strategy_coherence),
                    authority_request: merge_authority_request.as_ref(),
                    authority_receipt: merge_authority_receipt.as_ref(),
                    denial_class: AuthorityDenialBoundaryClass::MergeAuthorityRejection,
                    failure_kind: merge_error.kind(),
                    failure_digest: Some(&merge_failure_digest),
                    effect: Some(&merge_effect),
                    idempotence: Some(&merge_idempotence),
                    incoming_feedback_context: None,
                },
                unsafe_feedback: AuthorityDenialBoundaryFailureEvidence {
                    contract: Some(&unsafe_feedback_contract),
                    strategy_basis: unsafe_feedback_contract
                        .validated_declaration()
                        .strategy_basis(),
                    strategy_coherence: Some(&unsafe_feedback_strategy_coherence),
                    authority_request: None,
                    authority_receipt: None,
                    denial_class: AuthorityDenialBoundaryClass::UnsafeFeedbackPreauthority,
                    failure_kind: unsafe_feedback_error.kind(),
                    failure_digest: Some(&unsafe_feedback_failure_digest),
                    effect: Some(&unsafe_feedback_effect),
                    idempotence: Some(&unsafe_feedback_idempotence),
                    incoming_feedback_context: Some(&unsafe_feedback_drift_context),
                },
                contradictory_feedback: AuthorityDenialBoundaryFailureEvidence {
                    contract: Some(&unsafe_feedback_contract),
                    strategy_basis: unsafe_feedback_contract
                        .validated_declaration()
                        .strategy_basis(),
                    strategy_coherence: Some(&unsafe_feedback_strategy_coherence),
                    authority_request: None,
                    authority_receipt: None,
                    denial_class: AuthorityDenialBoundaryClass::ContradictoryFeedbackPreauthority,
                    failure_kind: contradictory_feedback_error.kind(),
                    failure_digest: Some(&contradictory_feedback_failure_digest),
                    effect: Some(&unsafe_feedback_effect),
                    idempotence: Some(&unsafe_feedback_idempotence),
                    incoming_feedback_context: Some(&contradictory_feedback_context),
                },
            },
        ),
        zero_residue_report: AuthorityDenialZeroResidueProof::no_authority_residue(),
        counter_snapshot,
    })
}
