use super::*;

#[test]
fn read_family_intent_common_path_helper_executes_through_canonical_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = read_runtime(request_execution);
    let mut workspace = WorthQueryWorkspace::new("intent-admission-read-dx", runtime)
        .expect("workspace should build");
    let family = identity_read_family(&mut workspace, "tasks");

    let result = workspace
        .read_family_intent(&family)
        .execute(request_execution)
        .expect("read common-path helper should execute");

    assert_eq!(
        result.receipt().decision_trace_envelope().map(trace_stages),
        Some(vec![
            WorthQueryIntentDecisionTraceStage::RawIntent,
            WorthQueryIntentDecisionTraceStage::Eligibility,
            WorthQueryIntentDecisionTraceStage::AdmittedDecision,
            WorthQueryIntentDecisionTraceStage::ExecutionHandoff,
            WorthQueryIntentDecisionTraceStage::ExecutionOutcome,
        ])
    );
    assert_eq!(
        result
            .receipt()
            .execution_provenance()
            .map(|provenance| provenance.family()),
        Some(WorthQueryIntentAdmissionFamily::ReadExecutionIntent)
    );
}

#[test]
fn read_family_intent_advanced_path_helper_exposes_request_eligibility_decision_and_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = read_runtime(request_execution);
    let mut workspace = WorthQueryWorkspace::new("intent-admission-read-advanced", runtime)
        .expect("workspace should build");
    let family = identity_read_family(&mut workspace, "tasks");

    let review = workspace
        .read_family_intent(&family)
        .review()
        .expect("advanced read path should review");
    let handoff = review
        .admitted_handoff()
        .expect("admitted read review should expose a handoff");

    assert_eq!(
        review.request().entrypoint(),
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteReadFamily
    );
    match review.decision() {
        WorthQueryIntentAdmissionDecision::Admitted(plan) => {
            assert_eq!(plan.decision_digest(), handoff.decision_digest());
        }
        other => panic!("expected admitted read review, got {other:?}"),
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
fn live_read_intent_common_path_helper_executes_through_canonical_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = read_runtime(request_execution);
    let mut workspace = WorthQueryWorkspace::new("intent-admission-live-read-dx", runtime)
        .expect("workspace should build");
    let live_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.table", |q| {
            q.from("Task")
                .select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                ])
                .order_by(
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                )
                .schema_basis("intent-admission-live-read")
        })
        .expect("live view should declare");

    let result = workspace
        .read_live_intent(&live_view)
        .execute(request_execution)
        .expect("live read common-path helper should execute");

    assert_eq!(
        result.receipt().decision_trace_envelope().map(trace_stages),
        Some(vec![
            WorthQueryIntentDecisionTraceStage::RawIntent,
            WorthQueryIntentDecisionTraceStage::Eligibility,
            WorthQueryIntentDecisionTraceStage::AdmittedDecision,
            WorthQueryIntentDecisionTraceStage::ExecutionHandoff,
            WorthQueryIntentDecisionTraceStage::ExecutionOutcome,
        ])
    );
    assert_eq!(
        result
            .receipt()
            .execution_provenance()
            .map(|provenance| provenance.entrypoint()),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteLiveRead)
    );
}

#[test]
fn live_read_intent_advanced_path_helper_exposes_request_eligibility_decision_and_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = read_runtime(request_execution);
    let mut workspace = WorthQueryWorkspace::new("intent-admission-live-read-advanced", runtime)
        .expect("workspace should build");
    let live_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.table", |q| {
            q.from("Task")
                .select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                ])
                .order_by(
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                )
                .schema_basis("intent-admission-live-read")
        })
        .expect("live view should declare");

    let review = workspace
        .read_live_intent(&live_view)
        .review()
        .expect("advanced live read path should review");
    let handoff = review
        .admitted_handoff()
        .expect("admitted live read review should expose a handoff");

    assert_eq!(
        review.request().entrypoint(),
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteLiveRead
    );
    match review.decision() {
        WorthQueryIntentAdmissionDecision::Admitted(plan) => {
            assert_eq!(plan.decision_digest(), handoff.decision_digest());
        }
        other => panic!("expected admitted live read review, got {other:?}"),
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
