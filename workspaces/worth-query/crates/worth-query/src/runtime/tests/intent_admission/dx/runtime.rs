use super::*;

#[test]
fn intent_common_path_helper_executes_through_canonical_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = intent_runtime_with_authority(request_execution, TestIntentAuthority);
    let receipt = runtime
        .intent(WorthQueryIntentDeclaration::strategy_commit(
            "helper-runtime-intent",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1"), ("title", "Intent committed title")]),
        ))
        .execute(request_execution)
        .expect("common-path helper should execute");

    assert_eq!(
        receipt.covered_entrypoint_label(),
        "WorthQueryRuntime::execute_intent"
    );
    assert_eq!(
        trace_stages(receipt.decision_trace_envelope()),
        vec![
            WorthQueryIntentDecisionTraceStage::RawIntent,
            WorthQueryIntentDecisionTraceStage::Eligibility,
            WorthQueryIntentDecisionTraceStage::AdmittedDecision,
            WorthQueryIntentDecisionTraceStage::ExecutionHandoff,
            WorthQueryIntentDecisionTraceStage::ExecutionOutcome,
        ]
    );
}

#[test]
fn intent_advanced_path_helper_exposes_request_eligibility_decision_and_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = intent_runtime_with_authority(request_execution, TestIntentAuthority);
    let review = runtime
        .intent(WorthQueryIntentDeclaration::strategy_commit(
            "advanced-helper-runtime-intent",
            "strategy.intent.reconcile",
            "1.0",
            "intent.reconcile.input.v1",
            test_intent_input([("entity", "task-1")]),
        ))
        .review()
        .expect("advanced path should review");

    assert_eq!(
        review.request().entrypoint().as_str(),
        "WorthQueryRuntime::execute_intent"
    );
    assert_eq!(
        review.request().request_digest(),
        review.eligibility().request().request_digest()
    );
    let decision = review.decision();
    let handoff = review
        .admitted_handoff()
        .expect("admitted review should expose a handoff");
    match decision {
        WorthQueryIntentAdmissionDecision::Admitted(plan) => {
            assert_eq!(plan.decision_digest(), handoff.decision_digest());
        }
        other => panic!("expected admitted review, got {other:?}"),
    }
    let consumer = review.consumer_inspection();
    assert_eq!(
        consumer.outcome_class(),
        WorthQueryIntentConsumerOutcomeClass::Admitted
    );
    assert_eq!(consumer.admission_family(), Some(review.request().family()));
    assert_eq!(
        consumer.covered_entrypoint(),
        Some(review.request().entrypoint())
    );
    assert_eq!(consumer.terminal_stage_label(), "admitted-decision");
    assert_eq!(consumer.terminal_cause(), "admitted_for_execution");
}

#[test]
fn effect_write_intent_common_path_helper_executes_through_canonical_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = intent_runtime_with_authority(request_execution, TestIntentAuthority);
    let live = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.effect-dx",
            task_live_request(),
            task_schema(),
        )
        .expect("live should declare");
    let effect = runtime
        .declare_effect::<WorthQueryUnrefinedLiveShape>(WorthQueryEffectDeclaration::write_intent(
            "effects.reconcile-dx",
            WorthQueryEffectTrigger::live_view(&live, test_aspect_touches(["title.value"])),
            "strategy.intent.reconcile",
        ))
        .expect("write-intent effect should declare");
    runtime
        .write(
            request_execution,
            test_update_string_aspect_command(
                crate::memory_workspace::admit_authored_entity_label("task-1"),
                "title.value",
                "title from helper",
            ),
        )
        .expect("write should queue pending effect intent");

    let receipt = runtime
        .next_effect_write_intent(&effect, "1.0", "effect.intent.input.v1")
        .execute(request_execution)
        .expect("effect common-path helper should execute");

    assert_eq!(
        receipt.intent_receipt().covered_entrypoint_label(),
        "WorthQueryRuntime::execute_next_effect_write_intent"
    );
    assert_eq!(receipt.effect_name(), "effects.reconcile-dx");
}

#[test]
fn write_intent_common_path_helper_executes_through_canonical_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = intent_runtime_with_authority(request_execution, TestIntentAuthority);
    let receipt = runtime
        .write_intent(test_update_string_aspect_command(
            crate::memory_workspace::admit_authored_entity_label("task-1"),
            "title.value",
            "title from write helper",
        ))
        .execute(request_execution)
        .expect("write common-path helper should execute");

    assert_eq!(
        receipt.covered_entrypoint_label(),
        Some("WorthQueryRuntime::write")
    );
    assert_eq!(
        receipt
            .decision_trace_envelope()
            .map(trace_stages)
            .unwrap_or_default(),
        vec![
            WorthQueryIntentDecisionTraceStage::RawIntent,
            WorthQueryIntentDecisionTraceStage::Eligibility,
            WorthQueryIntentDecisionTraceStage::AdmittedDecision,
            WorthQueryIntentDecisionTraceStage::ExecutionHandoff,
            WorthQueryIntentDecisionTraceStage::ExecutionOutcome,
        ]
    );
}

#[test]
fn write_intent_advanced_path_helper_exposes_request_eligibility_decision_and_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = intent_runtime_with_authority(request_execution, TestIntentAuthority);
    let review = runtime
        .write_intent(test_update_string_aspect_command(
            crate::memory_workspace::admit_authored_entity_label("task-1"),
            "title.value",
            "title from advanced write helper",
        ))
        .review(request_execution)
        .expect("advanced write path should review");

    assert_eq!(
        review.request().entrypoint().as_str(),
        "WorthQueryRuntime::write"
    );
    assert_eq!(
        review.request().request_digest(),
        review.eligibility().request().request_digest()
    );
    let handoff = review
        .admitted_handoff()
        .expect("admitted write review should expose a handoff");
    match review.decision() {
        WorthQueryIntentAdmissionDecision::Admitted(plan) => {
            assert_eq!(plan.decision_digest(), handoff.decision_digest());
        }
        other => panic!("expected admitted write review, got {other:?}"),
    }
    let consumer = review.consumer_inspection();
    assert_eq!(
        consumer.outcome_class(),
        WorthQueryIntentConsumerOutcomeClass::Admitted
    );
    assert_eq!(consumer.admission_family(), Some(review.request().family()));
    assert_eq!(
        consumer.covered_entrypoint(),
        Some(review.request().entrypoint())
    );
}

#[test]
fn canonical_admission_decision_round_trips_to_public_handoff_type() {
    let declaration = WorthQueryIntentDeclaration::strategy_commit(
        "canonical-admission-intent",
        "strategy.intent.reconcile",
        "1.0",
        "intent.reconcile.input.v1",
        test_intent_input([("entity", "task-1")]),
    );
    let request = crate::intent_admission::WorthQueryRawIntentAdmissionRequest::authoritative_runtime_entrypoint(
        declaration,
    )
    .expect("authoritative request should build");
    let decision = admit_runtime_intent_request(request);

    match decision {
        WorthQueryIntentAdmissionDecision::Admitted(plan) => {
            let handoff: WorthQueryAdmittedIntentExecutionHandoff = plan
                .into_execution_handoff()
                .expect("runtime admitted plan should still mint a handoff");
            assert_eq!(
                handoff.entrypoint(),
                WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteIntent
            );
            assert!(!handoff.handoff_digest().is_empty());
        }
        other => panic!("expected admitted decision, got {other:?}"),
    }
}

#[test]
fn advisory_review_data_preserves_non_panicking_trace_shape() {
    let declaration = WorthQueryIntentDeclaration::strategy_commit(
        "advisory-runtime-intent",
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
    let eligibility = crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(
        request.clone(),
    );
    let advisory = crate::intent_admission::WorthQueryIntentAdvisoryDecision::new(
        request.family(),
        request.entrypoint(),
        "materialized-detail-advisory",
        "full execution is intentionally deferred",
        request.request_digest(),
        eligibility.eligibility_digest(),
    );
    let review =
        crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData::from_decision(
            request,
            WorthQueryIntentAdmissionDecision::Advisory(advisory),
        );
    let decision_trace = review
        .decision_trace_envelope()
        .expect("advisory review should still produce a trace");

    assert_eq!(
        decision_trace.kind(),
        WorthQueryIntentDecisionTraceEnvelopeKind::AdvisoryStop
    );
    assert_eq!(
        trace_stages(decision_trace),
        vec![
            WorthQueryIntentDecisionTraceStage::RawIntent,
            WorthQueryIntentDecisionTraceStage::Eligibility,
            WorthQueryIntentDecisionTraceStage::AdvisoryStop,
        ]
    );
    match decision_trace.rows()[1].evidence() {
        WorthQueryIntentDecisionTraceEvidence::Eligibility(evidence) => {
            assert_eq!(
                evidence.support_posture(),
                WorthQueryIntentAdmissionSupportEligibility::Admitted
            );
            assert_eq!(
                evidence.routing_support_posture(),
                WorthQueryIntentAdmissionRoutingSupportEligibility::CoveredExecutionSeam(
                    crate::facade::runtime::WorthQueryIntentAdmissionExecutionSeam::BackendIntentAuthorityRoute
                )
            );
            assert_eq!(
                decision_trace.rows()[1].artifact_digest(),
                evidence.eligibility_digest()
            );
        }
        other => panic!("expected eligibility evidence on advisory trace, got {other:?}"),
    }
    match decision_trace.rows()[2].evidence() {
        WorthQueryIntentDecisionTraceEvidence::NonAdmittedDecision { decision_digest } => {
            match review.decision() {
                WorthQueryIntentAdmissionDecision::Advisory(advisory) => {
                    assert_eq!(decision_digest, advisory.decision_digest());
                }
                other => panic!("expected advisory decision, got {other:?}"),
            }
        }
        other => panic!("expected non-admitted decision evidence, got {other:?}"),
    }
}

#[test]
fn advisory_consumer_lane_stays_on_shared_lattice_surface() {
    let declaration = WorthQueryIntentDeclaration::strategy_commit(
        "advisory-consumer-runtime-intent",
        "strategy.intent.reconcile",
        "1.0",
        "intent.reconcile.input.v1",
        test_intent_input([("entity", "task-1")]),
    );
    let request = crate::intent_admission::WorthQueryRawIntentAdmissionRequest::authoritative_runtime_entrypoint(
        declaration,
    )
    .expect("authoritative request should build");
    let eligibility = crate::intent_admission::WorthQueryIntentAdmissionEligibility::from_request(
        request.clone(),
    );
    let advisory = crate::intent_admission::WorthQueryIntentAdvisoryDecision::new(
        request.family(),
        request.entrypoint(),
        "materialized-detail-advisory",
        "full execution is intentionally deferred",
        request.request_digest(),
        eligibility.eligibility_digest(),
    );
    let review =
        crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData::from_decision(
            request,
            WorthQueryIntentAdmissionDecision::Advisory(advisory),
        );
    let consumer = crate::facade::runtime::WorthQueryIntentConsumerInspection::from_review(
        review.request().intent_name(),
        review.decision(),
        review.request().family(),
        review.request().entrypoint(),
        review.decision_trace_envelope(),
    );

    assert_eq!(
        consumer.outcome_class(),
        WorthQueryIntentConsumerOutcomeClass::Advisory
    );
    assert_eq!(
        consumer.decision_trace_envelope_kind(),
        Some(WorthQueryIntentDecisionTraceEnvelopeKind::AdvisoryStop)
    );
    assert_eq!(consumer.admission_family(), Some(review.request().family()));
    assert_eq!(
        consumer.covered_entrypoint(),
        Some(review.request().entrypoint())
    );
    assert_eq!(
        consumer.terminal_stage(),
        Some(WorthQueryIntentDecisionTraceStage::AdvisoryStop)
    );
    assert_eq!(consumer.terminal_cause(), "materialized-detail-advisory");
    assert_eq!(
        consumer.terminal_detail(),
        "full execution is intentionally deferred"
    );
    assert_eq!(
        consumer.decision_trace_digest(),
        review
            .decision_trace_envelope()
            .map(|trace| trace.trace_digest())
    );
    assert_eq!(consumer.execution_provenance_chain_digest(), None);
}
