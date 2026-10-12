use super::*;

#[test]
fn denied_intent_stops_before_backend_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let attempted = Rc::new(Cell::new(0));
    let mut runtime = intent_runtime_with_authority(
        request_execution,
        CountingIntentAuthority {
            attempted: attempted.clone(),
        },
    );
    let declaration = WorthQueryIntentDeclaration::strategy_commit(
        "denied-runtime-intent",
        "strategy.intent.reconcile",
        "1.0",
        "intent.reconcile.input.v1",
        test_intent_input([("entity", "task-1")]),
    )
    .with_source_lane(WorthQueryIntentSourceLane::EffectTriggered);

    let error = runtime
        .execute_intent(request_execution, declaration)
        .expect_err("mismatched source lane should deny");

    assert_eq!(attempted.get(), 0);
    match error {
        WorthQueryRuntimeError::IntentCommitDenied {
            stage, evidence, ..
        } => {
            assert_eq!(stage, "source-lane-admission");
            let decision_trace = evidence
                .decision_trace_envelope()
                .expect("covered denial should expose a decision trace");
            assert_eq!(
                decision_trace.kind(),
                WorthQueryIntentDecisionTraceEnvelopeKind::ViolationStop
            );
            assert_eq!(
                trace_stages(decision_trace),
                vec![
                    WorthQueryIntentDecisionTraceStage::RawIntent,
                    WorthQueryIntentDecisionTraceStage::Eligibility,
                    WorthQueryIntentDecisionTraceStage::ViolationStop,
                ]
            );
            match decision_trace.rows()[1].evidence() {
                WorthQueryIntentDecisionTraceEvidence::Eligibility(evidence) => {
                    assert_eq!(
                        evidence.capability_posture(),
                        WorthQueryIntentAdmissionCapabilityEligibility::Violation {
                            stage: "source-lane-admission",
                            detail:
                                "covered-runtime-entrypoint-rejects-effect-triggered-source-lane",
                        }
                    );
                    assert_eq!(
                        evidence.source_lane_posture(),
                        WorthQueryIntentAdmissionSourceLaneEligibility::Mismatch {
                            expected: WorthQueryIntentSourceLane::UserAuthored,
                            actual: WorthQueryIntentSourceLane::EffectTriggered,
                        }
                    );
                }
                other => panic!("expected structured eligibility evidence, got {other:?}"),
            }
        }
        other => panic!("expected intent denial, got {other:?}"),
    }
}

#[test]
fn execution_denial_keeps_admission_and_execution_provenance_in_trace() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime =
        intent_runtime_with_authority(request_execution, InvariantViolationIntentAuthority);
    let error = runtime
        .execute_intent(
            request_execution,
            WorthQueryIntentDeclaration::strategy_commit(
                "traceable-invariant-denial",
                "strategy.intent.reconcile",
                "1.0",
                "intent.reconcile.input.v1",
                test_intent_input([("entity", "task-1"), ("dependency", "cycle")]),
            ),
        )
        .expect_err("invariant violation must deny");

    match error {
        WorthQueryRuntimeError::IntentCommitDenied { evidence, .. } => {
            let decision_trace = evidence
                .decision_trace_envelope()
                .expect("covered execution denial should expose a decision trace");
            assert_eq!(
                decision_trace.kind(),
                WorthQueryIntentDecisionTraceEnvelopeKind::ViolationStop
            );
            assert_eq!(
                trace_stages(decision_trace),
                vec![
                    WorthQueryIntentDecisionTraceStage::RawIntent,
                    WorthQueryIntentDecisionTraceStage::Eligibility,
                    WorthQueryIntentDecisionTraceStage::AdmittedDecision,
                    WorthQueryIntentDecisionTraceStage::ExecutionHandoff,
                    WorthQueryIntentDecisionTraceStage::ExecutionOutcome,
                    WorthQueryIntentDecisionTraceStage::ViolationStop,
                ]
            );
            assert_eq!(
                decision_trace.trace_digest(),
                evidence
                    .decision_trace_envelope()
                    .expect("trace should still be present")
                    .trace_digest()
            );
            let consumer = evidence.consumer_inspection();
            assert_eq!(
                consumer.outcome_class(),
                WorthQueryIntentConsumerOutcomeClass::Violation
            );
            assert_eq!(
                consumer.decision_trace_envelope_kind(),
                Some(WorthQueryIntentDecisionTraceEnvelopeKind::ViolationStop)
            );
            assert_eq!(
                consumer.execution_provenance_chain_digest(),
                evidence
                    .execution_provenance()
                    .map(|provenance| provenance.execution_provenance_chain_digest())
            );
            assert_eq!(
                consumer.terminal_stage(),
                Some(WorthQueryIntentDecisionTraceStage::ViolationStop)
            );
        }
        other => panic!("expected invariant-denial evidence, got {other:?}"),
    }
}
