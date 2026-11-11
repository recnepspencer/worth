use super::support::*;

#[test]
fn runtime_executes_writeback_through_bound_authority() {
    let runtime = runtime_with_writeback_authority(BridgeRuntimePolicy::default());
    let lowered_policy = lowered_policy(&runtime);
    let declaration = writeback_declaration(
        BridgeWritebackDeclarationIdentity::admit_bridge_owned("writeback:authority-execution"),
        BridgeRequestKind::Authoritative,
        BridgeWritebackRequestMode::WritebackCapable,
        "authority-execution",
    );
    let contract = runtime
        .admit_writeback_declaration(declaration, &lowered_policy)
        .expect("writeback declaration should admit");
    let effect = runtime.lower_writeback_effect(
        &contract,
        &causality_basis(
            BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:authority-execution"),
            "commit-a",
        ),
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:authority-execution"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "authoritative-upsert",
        ),
    );
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &truth_state_basis(&effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned("idempotence:authority-execution"),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    let (outcome, receipt) = runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect("bound writeback authority should execute");

    assert_eq!(
        receipt.outcome_class(),
        crate::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit
    );
    assert!(receipt
        .authoritative_artifact_digest()
        .starts_with("truth-writeback-authoritative-artifact:sha256:"));
    assert_eq!(receipt.failure_class(), None);
    assert!(outcome
        .digest()
        .starts_with("bridge-writeback-authority-outcome:sha256:"));
}

#[test]
fn mutation_subject_cannot_authorize_a_different_effect_intent() {
    let runtime = runtime_with_writeback_authority(BridgeRuntimePolicy::default());
    let lowered_policy = lowered_policy(&runtime);
    let contract = runtime
        .admit_writeback_declaration(
            writeback_declaration(
                BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "writeback:subject-effect-mismatch",
                ),
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                "subject-effect-mismatch",
            ),
            &lowered_policy,
        )
        .expect("writeback declaration should admit");
    let subject_intent = writeback_effect_intent(
        BridgeWritebackEffectClass::ProjectedStateDiff,
        "subject-patch",
    );
    let subject = crate::facade::BridgeMutationSubject::from_effect_intent_and_touches(
        crate::facade::BridgeMutationSubjectTarget::new(
            "Task",
            crate::facade::RelationalBridgeRecordIdentityParts::entity(1, 7, 0),
            crate::facade::BridgeMutationSubjectKind::Updated,
        ),
        &subject_intent,
        [crate::facade::BridgeMutationSubjectTouch::whole_aspect(
            worth_foundational::facade::AspectKey::new("bridge.writeback.projected-state-diff")
                .expect("static writeback aspect key is valid"),
        )],
    )
    .expect("subject should cover its own concrete patch");
    let causality = causality_basis(
        BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:subject-effect-mismatch"),
        "subject-effect-mismatch",
    )
    .bind_mutation_subject(subject);
    let effect = runtime.lower_writeback_effect(
        &contract,
        &causality,
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:subject-effect-mismatch"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "different-effect-patch",
        ),
    );
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &truth_state_basis(&effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:subject-effect-mismatch",
        ),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    let error = runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect_err("mismatched mutation subject must deny before authority execution");

    assert_eq!(
        error.kind(),
        BridgeWritebackErrorKind::CausalityEffectMismatch
    );
    assert!(matches!(
        runtime.diagnostics().last_writeback_execution_record(),
        Err(worth_foundational::facade::DiagnosticRecordAbsence::NeverRetained)
    ));
}

#[test]
fn native_writeback_execution_retention_expires_without_losing_the_commit_receipt() {
    let runtime = runtime_with_writeback_authority(
        BridgeRuntimePolicy::development().with_route_record_limit(1),
    );
    let lowered_policy = lowered_policy(&runtime);
    let contract = runtime
        .admit_writeback_declaration(
            writeback_declaration(
                BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "writeback:execution-record-success",
                ),
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                "execution-record-success",
            ),
            &lowered_policy,
        )
        .expect("writeback declaration should admit");
    let effect = runtime.lower_writeback_effect(
        &contract,
        &causality_basis(
            BridgeWritebackCausalityIdentity::admit_bridge_owned(
                "causality:execution-record-success",
            ),
            "commit-a",
        ),
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:execution-record-success"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "execution-record-success",
        ),
    );
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &truth_state_basis(&effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:execution-record-success",
        ),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    let (outcome, receipt) = runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect("authority execution should succeed");
    let record = runtime
        .diagnostics()
        .last_writeback_execution_record()
        .expect("runtime should retain a native writeback execution record");

    assert_eq!(record.contract_digest(), contract.digest());
    assert_eq!(record.writeback_effect_artifact_digest(), effect.digest());
    assert_eq!(record.effect_intent_digest(), effect.effect_intent_digest());
    assert_eq!(
        record.effect_intent_patch_canonical_basis(),
        effect.effect_intent().patch_canonical_basis()
    );
    assert_eq!(record.causality_digest(), effect.causality_digest());
    assert_eq!(record.outcome_digest(), Some(outcome.digest()));
    assert_eq!(
        record.outcome_class(),
        Some(crate::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit)
    );
    assert_eq!(record.request_digest(), Some(receipt.request_digest()));
    assert_eq!(record.receipt_digest(), Some(receipt.digest()));
    assert_eq!(
        record
            .authority_request()
            .expect("execution record should retain authority request")
            .effect_intent(),
        effect.effect_intent()
    );
    assert_eq!(
        record
            .authority_receipt()
            .expect("execution record should retain authority receipt")
            .effect_intent(),
        effect.effect_intent()
    );
    assert_eq!(record.failure_class(), None);
    assert_eq!(record.counters().writeback_request_count(), 1);
    assert_eq!(record.counters().writeback_commit_count(), 1);
    assert_eq!(record.counters().writeback_failure_count(), 0);
    super::diagnostic_retention::execution_expiry(&runtime.diagnostics, &record, &receipt);
}

#[test]
fn writeback_mutation_authority_bundle_preserves_causality_and_provenance() {
    let runtime = runtime_with_writeback_authority(BridgeRuntimePolicy::development());
    let lowered_policy = lowered_policy(&runtime);
    let contract = runtime
        .admit_writeback_declaration(
            writeback_declaration(
                BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "writeback:mutation-authority-bundle",
                ),
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                "mutation-authority-bundle",
            ),
            &lowered_policy,
        )
        .expect("writeback declaration should admit");
    let effect_intent = writeback_effect_intent(
        BridgeWritebackEffectClass::ProjectedStateDiff,
        "mutation-authority-bundle",
    );
    let causality = mutation_causality_basis(
        BridgeWritebackCausalityIdentity::admit_bridge_owned("causality:mutation-authority-bundle"),
        "mutation-authority-bundle",
        &effect_intent,
    );
    let effect = runtime.lower_writeback_effect(
        &contract,
        &causality,
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:mutation-authority-bundle"),
        effect_intent,
    );
    let feedback = crate::facade::BridgeWritebackFeedbackProvenance::new(&effect);
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &truth_state_basis(&effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:mutation-authority-bundle",
        ),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    let bundle = runtime
        .execute_writeback_mutation_authority(&contract, &effect, &idempotence, &causality)
        .expect("one successful writeback chain should mint mutation authority");
    let execution_record = runtime
        .diagnostics()
        .last_writeback_execution_record()
        .expect("runtime should retain a native writeback execution record");

    assert_eq!(bundle.causality().causality_digest(), causality.digest());
    assert_eq!(
        bundle.causality().truth_trigger_digest(),
        causality.truth_trigger_digest()
    );
    assert_eq!(
        bundle.provenance().feedback_provenance_digest(),
        feedback.digest()
    );
    assert_eq!(
        bundle.provenance().execution_record_digest(),
        execution_record.digest()
    );
    assert_eq!(
        bundle.provenance().authoritative_artifact_digest(),
        execution_record
            .authority_receipt()
            .map(|receipt| receipt.authoritative_artifact_digest())
    );
    assert_eq!(
        bundle.provenance().receipt_digest(),
        execution_record.receipt_digest()
    );
    assert_eq!(
        bundle
            .provenance()
            .authority_request()
            .expect("mutation provenance should retain authority request")
            .effect_intent(),
        effect.effect_intent()
    );
    assert_eq!(
        bundle
            .provenance()
            .authority_receipt()
            .expect("mutation provenance should retain authority receipt")
            .effect_intent(),
        effect.effect_intent()
    );
    assert_eq!(
        bundle.provenance().outcome_class(),
        Some(crate::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit)
    );
}

#[test]
fn writeback_mutation_provenance_bundle_preserves_rejection_class() {
    let runtime = runtime_with_custom_writeback_authority(
        BridgeRuntimePolicy::development(),
        RejectingWritebackAuthority {
            failure_class: BridgeWritebackFailureClass::StrategyFailed,
        },
    );
    let lowered_policy = lowered_policy(&runtime);
    let contract = runtime
        .admit_writeback_declaration(
            writeback_declaration(
                BridgeWritebackDeclarationIdentity::admit_bridge_owned(
                    "writeback:mutation-authority-rejection",
                ),
                BridgeRequestKind::Authoritative,
                BridgeWritebackRequestMode::WritebackCapable,
                "mutation-authority-rejection",
            ),
            &lowered_policy,
        )
        .expect("writeback declaration should admit");
    let causality = causality_basis(
        BridgeWritebackCausalityIdentity::admit_bridge_owned(
            "causality:mutation-authority-rejection",
        ),
        "mutation-authority-rejection",
    );
    let effect = runtime.lower_writeback_effect(
        &contract,
        &causality,
        BridgeWritebackEffectIdentity::admit_bridge_owned("effect:mutation-authority-rejection"),
        writeback_effect_intent(
            BridgeWritebackEffectClass::ProjectedStateDiff,
            "mutation-authority-rejection",
        ),
    );
    let feedback = crate::facade::BridgeWritebackFeedbackProvenance::new(&effect);
    let idempotence = runtime.classify_writeback_idempotence(
        &effect,
        &lowered_policy,
        &truth_state_basis(&effect),
        BridgeWritebackIdempotenceIdentity::admit_bridge_owned(
            "idempotence:mutation-authority-rejection",
        ),
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    );

    let error = runtime
        .execute_writeback_authority(&contract, &effect, &idempotence)
        .expect_err("authority execution should surface the typed rejection");
    let execution_record = runtime
        .diagnostics()
        .last_writeback_execution_record()
        .expect("runtime should retain a rejection execution record");
    let provenance = crate::facade::BridgeMutationProvenanceBundle::from_writeback_artifacts(
        &effect,
        &feedback,
        &execution_record,
        None,
    );

    assert_eq!(error.kind(), BridgeWritebackErrorKind::StrategyFailed);
    assert_eq!(provenance.outcome_class(), None);
    assert_eq!(
        provenance.failure_class(),
        Some(BridgeWritebackFailureClass::StrategyFailed)
    );
    assert_eq!(
        provenance.receipt_digest(),
        execution_record.receipt_digest()
    );
}
