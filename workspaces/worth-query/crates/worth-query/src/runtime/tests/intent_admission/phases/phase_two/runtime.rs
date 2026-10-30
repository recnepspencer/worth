use super::*;

#[test]
fn authoritative_runtime_floor_eligibility_carries_closed_pre_execution_facts() {
    let declaration = WorthQueryIntentDeclaration::strategy_commit(
        "eligibility-authoritative-intent",
        "strategy.intent.reconcile",
        "1.0",
        "intent.reconcile.input.v1",
        test_intent_input([("entity", "task-1")]),
    );
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::authoritative_runtime_entrypoint(
            declaration,
        )
        .expect("authoritative request should build");
    let eligibility =
        crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(request);

    assert_eq!(
        eligibility.support_posture(),
        WorthQueryIntentAdmissionSupportEligibility::Admitted
    );
    assert_eq!(
        eligibility.capability_posture(),
        WorthQueryIntentAdmissionCapabilityEligibility::Admitted
    );
    assert_eq!(
        eligibility.policy_posture(),
        WorthQueryIntentAdmissionPolicyEligibility::NotApplicableForRuntimeIntentFloor
    );
    assert_eq!(
        eligibility.basis_posture(),
        WorthQueryIntentAdmissionBasisEligibility::NotApplicableForRuntimeIntentFloor
    );
    assert_eq!(
        eligibility.invariant_posture(),
        WorthQueryIntentAdmissionInvariantEligibility::PreExecutionAuthorityRequired
    );
    assert_eq!(
        eligibility.projection_source_posture(),
        WorthQueryIntentAdmissionProjectionSourceEligibility::NotApplicableForRuntimeIntentFloor
    );
    assert_eq!(
        eligibility.routing_support_posture(),
        WorthQueryIntentAdmissionRoutingSupportEligibility::CoveredExecutionSeam(
            crate::facade::runtime::WorthQueryIntentAdmissionExecutionSeam::BackendIntentAuthorityRoute
        )
    );
    assert_eq!(
        eligibility.source_lane_posture(),
        WorthQueryIntentAdmissionSourceLaneEligibility::MatchesExpected(
            WorthQueryIntentSourceLane::UserAuthored
        )
    );
    assert_eq!(
        eligibility.authority_lane_posture(),
        WorthQueryIntentAdmissionAuthorityLaneEligibility::MatchesExpected(
            WorthQueryAuthorityLane::AuthoritativeTruth
        )
    );
    assert_eq!(
        eligibility.pre_decision_posture(),
        WorthQueryIntentAdmissionPreDecisionPosture::Admitted
    );
    assert_eq!(
        eligibility.admitted_execution_seam(),
        Some(
            crate::facade::runtime::WorthQueryIntentAdmissionExecutionSeam::BackendIntentAuthorityRoute
        )
    );
}

#[test]
fn effect_runtime_floor_eligibility_carries_closed_pre_execution_facts() {
    let declaration = WorthQueryIntentDeclaration::strategy_commit(
        "eligibility-effect-intent",
        "strategy.intent.reconcile",
        "1.0",
        "intent.reconcile.input.v1",
        test_intent_input([("entity", "task-1")]),
    )
    .with_source_lane(WorthQueryIntentSourceLane::EffectTriggered);
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::effect_runtime_entrypoint(
            declaration,
        )
        .expect("effect request should build");
    let eligibility =
        crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(request);

    assert_eq!(
        eligibility.support_posture(),
        WorthQueryIntentAdmissionSupportEligibility::Admitted
    );
    assert_eq!(
        eligibility.capability_posture(),
        WorthQueryIntentAdmissionCapabilityEligibility::Admitted
    );
    assert_eq!(
        eligibility.source_lane_posture(),
        WorthQueryIntentAdmissionSourceLaneEligibility::MatchesExpected(
            WorthQueryIntentSourceLane::EffectTriggered
        )
    );
    assert_eq!(
        eligibility.authority_lane_posture(),
        WorthQueryIntentAdmissionAuthorityLaneEligibility::MatchesExpected(
            WorthQueryAuthorityLane::AuthoritativeTruth
        )
    );
    assert_eq!(
        eligibility.admitted_execution_seam(),
        Some(
            crate::facade::runtime::WorthQueryIntentAdmissionExecutionSeam::BackendIntentAuthorityRoute
        )
    );
}

#[test]
fn mismatched_runtime_floor_eligibility_stops_as_violation() {
    let declaration = WorthQueryIntentDeclaration::strategy_commit(
        "eligibility-mismatch-intent",
        "strategy.intent.reconcile",
        "1.0",
        "intent.reconcile.input.v1",
        test_intent_input([("entity", "task-1")]),
    )
    .with_source_lane(WorthQueryIntentSourceLane::EffectTriggered);
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::authoritative_runtime_entrypoint(
            declaration,
        )
        .expect("authoritative request should build");
    let eligibility = crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(
        request.clone(),
    );
    let decision = admit_runtime_intent_request(request);

    assert_eq!(
        eligibility.capability_posture(),
        WorthQueryIntentAdmissionCapabilityEligibility::Violation {
            stage: "source-lane-admission",
            detail: "covered-runtime-entrypoint-rejects-effect-triggered-source-lane",
        }
    );
    assert_eq!(
        eligibility.source_lane_posture(),
        WorthQueryIntentAdmissionSourceLaneEligibility::Mismatch {
            expected: WorthQueryIntentSourceLane::UserAuthored,
            actual: WorthQueryIntentSourceLane::EffectTriggered,
        }
    );
    assert_eq!(
        eligibility.pre_decision_posture(),
        WorthQueryIntentAdmissionPreDecisionPosture::Violation {
            stage: "source-lane-admission",
            message: "covered-runtime-entrypoint-rejects-effect-triggered-source-lane",
        }
    );
    match decision {
        WorthQueryIntentAdmissionDecision::Violation(violation) => {
            assert_eq!(violation.stage(), "source-lane-admission");
            assert_eq!(
                violation.message(),
                "covered-runtime-entrypoint-rejects-effect-triggered-source-lane"
            );
        }
        other => panic!("expected violation decision, got {other:?}"),
    }
}

#[test]
fn authority_lane_mismatch_stops_as_violation() {
    let declaration = WorthQueryIntentDeclaration::strategy_commit(
        "eligibility-authority-mismatch-intent",
        "strategy.intent.reconcile",
        "1.0",
        "intent.reconcile.input.v1",
        test_intent_input([("entity", "task-1")]),
    )
    .with_target_lane(WorthQueryAuthorityLane::PreviewTruth);
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::authoritative_runtime_entrypoint(
            declaration,
        )
        .expect("authoritative request should build");
    let eligibility = crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(
        request.clone(),
    );
    let decision = admit_runtime_intent_request(request);

    assert_eq!(
        eligibility.capability_posture(),
        WorthQueryIntentAdmissionCapabilityEligibility::Violation {
            stage: "authority-admission",
            detail: "covered-runtime-entrypoint-rejects-preview-truth-target",
        }
    );
    assert_eq!(
        eligibility.authority_lane_posture(),
        WorthQueryIntentAdmissionAuthorityLaneEligibility::Mismatch {
            expected: WorthQueryAuthorityLane::AuthoritativeTruth,
            actual: WorthQueryAuthorityLane::PreviewTruth,
        }
    );
    assert_eq!(
        eligibility.pre_decision_posture(),
        WorthQueryIntentAdmissionPreDecisionPosture::Violation {
            stage: "authority-admission",
            message: "covered-runtime-entrypoint-rejects-preview-truth-target",
        }
    );
    match decision {
        WorthQueryIntentAdmissionDecision::Violation(violation) => {
            assert_eq!(violation.stage(), "authority-admission");
            assert_eq!(
                violation.message(),
                "covered-runtime-entrypoint-rejects-preview-truth-target"
            );
        }
        other => panic!("expected violation decision, got {other:?}"),
    }
}

#[test]
fn eligibility_digest_changes_when_pre_decision_fact_meaning_changes() {
    let source_lane_violation = crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::authoritative_runtime_entrypoint(
            WorthQueryIntentDeclaration::strategy_commit(
                "eligibility-source-digest-intent",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "task-1")]),
            )
            .with_source_lane(WorthQueryIntentSourceLane::EffectTriggered),
        )
        .expect("authoritative request should build"),
    );
    let authority_lane_violation = crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::authoritative_runtime_entrypoint(
            WorthQueryIntentDeclaration::strategy_commit(
                "eligibility-authority-digest-intent",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "task-1")]),
            )
            .with_target_lane(WorthQueryAuthorityLane::PreviewTruth),
        )
        .expect("authoritative request should build"),
    );

    assert_ne!(
        source_lane_violation.eligibility_digest(),
        authority_lane_violation.eligibility_digest()
    );
}
