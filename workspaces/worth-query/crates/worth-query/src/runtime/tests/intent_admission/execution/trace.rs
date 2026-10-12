use super::*;

#[test]
fn admitted_intent_receipt_exposes_linear_decision_trace() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = intent_runtime_with_authority(request_execution, TestIntentAuthority);
    let receipt = runtime
        .execute_intent(
            request_execution,
            WorthQueryIntentDeclaration::strategy_commit(
                "traceable-runtime-intent",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "task-1"), ("title", "Intent committed title")]),
            ),
        )
        .expect("intent should execute");
    let decision_trace = receipt.decision_trace_envelope();

    assert_eq!(
        decision_trace.kind(),
        WorthQueryIntentDecisionTraceEnvelopeKind::AdmittedExecution
    );
    assert_eq!(
        trace_stages(decision_trace),
        vec![
            WorthQueryIntentDecisionTraceStage::RawIntent,
            WorthQueryIntentDecisionTraceStage::Eligibility,
            WorthQueryIntentDecisionTraceStage::AdmittedDecision,
            WorthQueryIntentDecisionTraceStage::ExecutionHandoff,
            WorthQueryIntentDecisionTraceStage::ExecutionOutcome,
        ]
    );
    assert_eq!(decision_trace.family().as_str(), receipt.admission_family());
    assert_eq!(
        decision_trace.entrypoint().as_str(),
        receipt.covered_entrypoint_label()
    );
    match decision_trace.rows()[1].evidence() {
        WorthQueryIntentDecisionTraceEvidence::Eligibility(evidence) => {
            assert_eq!(
                decision_trace.rows()[1].evidence_owner(),
                WorthQueryIntentDecisionTraceEvidenceOwner::QueryIntentEligibility
            );
            assert_eq!(
                evidence.support_posture(),
                WorthQueryIntentAdmissionSupportEligibility::Admitted
            );
            assert_eq!(
                evidence.capability_posture(),
                WorthQueryIntentAdmissionCapabilityEligibility::Admitted
            );
            assert_eq!(
                evidence.policy_posture(),
                WorthQueryIntentAdmissionPolicyEligibility::NotApplicableForRuntimeIntentFloor
            );
            assert_eq!(
                evidence.basis_posture(),
                WorthQueryIntentAdmissionBasisEligibility::NotApplicableForRuntimeIntentFloor
            );
            assert_eq!(
                evidence.invariant_posture(),
                WorthQueryIntentAdmissionInvariantEligibility::PreExecutionAuthorityRequired
            );
            assert_eq!(
                evidence.projection_source_posture(),
                WorthQueryIntentAdmissionProjectionSourceEligibility::NotApplicableForRuntimeIntentFloor
            );
            assert_eq!(
                evidence.routing_support_posture(),
                WorthQueryIntentAdmissionRoutingSupportEligibility::CoveredExecutionSeam(
                    crate::facade::runtime::WorthQueryIntentAdmissionExecutionSeam::BackendIntentAuthorityRoute
                )
            );
            assert_eq!(
                evidence.source_lane_posture(),
                WorthQueryIntentAdmissionSourceLaneEligibility::MatchesExpected(
                    WorthQueryIntentSourceLane::UserAuthored
                )
            );
            assert_eq!(
                evidence.authority_lane_posture(),
                WorthQueryIntentAdmissionAuthorityLaneEligibility::MatchesExpected(
                    WorthQueryAuthorityLane::AuthoritativeTruth
                )
            );
        }
        other => panic!("expected structured eligibility evidence, got {other:?}"),
    }
    assert!(!decision_trace.trace_digest().is_empty());
    let consumer = receipt.consumer_inspection();
    assert_eq!(
        consumer.outcome_class(),
        WorthQueryIntentConsumerOutcomeClass::Admitted
    );
    assert_eq!(
        consumer.decision_trace_envelope_kind(),
        Some(WorthQueryIntentDecisionTraceEnvelopeKind::AdmittedExecution)
    );
    assert_eq!(
        consumer.execution_provenance_chain_digest(),
        Some(receipt.execution_provenance_chain_digest())
    );
    assert_eq!(
        consumer.terminal_stage(),
        Some(WorthQueryIntentDecisionTraceStage::ExecutionOutcome)
    );
}

#[test]
fn effect_triggered_trace_eligibility_preserves_write_adjacent_trigger_proof() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = intent_runtime_with_authority(request_execution, TestIntentAuthority);
    let origin_identity = test_write_adjacent_origin_identity(
        WorthQueryEffectWriteAdjacentTriggerClass::RemaskDrift,
        "remask-drift:cause:task-title",
    );
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.trace-follow-on",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(
            WorthQueryEffectDeclaration::write_intent(
                "effects.trace-follow-on",
                WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title.value"])),
                "strategy.intent.reconcile",
            )
            .with_write_adjacent_trigger(
                WorthQueryEffectWriteAdjacentTriggerClass::RemaskDrift,
                origin_identity.clone(),
            ),
        )
        .expect("remask drift effect should declare");

    runtime
        .write(
            request_execution,
            test_update_string_aspect_command(
                crate::memory_workspace::admit_authored_entity_label("task-1"),
                "title.value",
                "title from remask drift",
            ),
        )
        .expect("write should queue pending intent");

    let receipt = runtime
        .next_effect_write_intent(&effect, "1.0", "effect.intent.input.v1")
        .execute(request_execution)
        .expect("effect-triggered intent should execute");
    let decision_trace = receipt.intent_receipt().decision_trace_envelope();

    match decision_trace.rows()[1].evidence() {
        WorthQueryIntentDecisionTraceEvidence::Eligibility(evidence) => {
            let trigger = evidence
                .write_adjacent_trigger()
                .expect("effect-triggered trace should retain write-adjacent trigger proof");
            assert_eq!(
                trigger.class(),
                WorthQueryEffectWriteAdjacentTriggerClass::RemaskDrift
            );
            assert_eq!(trigger.origin_evidence_identity(), &origin_identity);
        }
        other => panic!("expected structured eligibility evidence, got {other:?}"),
    }
}
