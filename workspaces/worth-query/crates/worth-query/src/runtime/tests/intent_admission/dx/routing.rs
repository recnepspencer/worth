use super::*;

#[test]
fn existing_truth_probe_intent_common_path_executes_through_canonical_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("intent-admission-probe-dx")
        .expect("workspace should open");
    let binding = seeded_probe_binding(request_execution, &mut workspace);
    let request = WorthQueryExistingTruthProbeRequest::new(
        binding,
        test_aspect_touches(["identity.id", "title.value"]),
    )
    .expect("probe request should build");
    let runtime = workspace.into_runtime();

    let result = runtime
        .probe_existing_intent(request)
        .execute(request_execution)
        .expect("probe common-path helper should execute");

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
        Some(WorthQueryIntentAdmissionFamily::LowerRuntimeCapabilityRoutingIntent)
    );
}

#[test]
fn existing_truth_probe_intent_advanced_path_exposes_request_eligibility_decision_and_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("intent-admission-probe-advanced")
        .expect("workspace should open");
    let binding = seeded_probe_binding(request_execution, &mut workspace);
    let request = WorthQueryExistingTruthProbeRequest::new(
        binding,
        test_aspect_touches(["identity.id", "title.value"]),
    )
    .expect("probe request should build");
    let runtime = workspace.into_runtime();

    let review = runtime
        .probe_existing_intent(request)
        .review()
        .expect("advanced probe path should review");
    let handoff = review
        .admitted_handoff()
        .expect("admitted probe review should expose a handoff");

    assert_eq!(
        review.request().entrypoint(),
        WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteExistingTruthProbeRouting
    );
    match review.decision() {
        WorthQueryIntentAdmissionDecision::Admitted(plan) => {
            assert_eq!(plan.decision_digest(), handoff.decision_digest());
        }
        other => panic!("expected admitted probe review, got {other:?}"),
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
fn workspace_existing_truth_probe_intent_common_path_executes_through_canonical_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("intent-admission-workspace-probe-dx")
        .expect("workspace should open");
    let binding = seeded_probe_binding(request_execution, &mut workspace);
    let request = WorthQueryExistingTruthProbeRequest::new(
        binding,
        test_aspect_touches(["identity.id", "title.value"]),
    )
    .expect("probe request should build");

    let result = workspace
        .probe_existing_intent(request)
        .execute(request_execution)
        .expect("workspace probe common-path helper should execute");

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
        Some(WorthQueryIntentAdmissionFamily::LowerRuntimeCapabilityRoutingIntent)
    );
}

fn seeded_probe_binding(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
) -> WorthQueryExistingTruthTargetBinding {
    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Seed title"),
            )
        })
        .expect("seed insert should execute");
    workspace
        .bind_existing_entity(
            WorthQueryExistingEntityTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                seed.deltas()[0].entity_identity.clone(),
            )
            .expect("existing entity target should build")
            .in_target_collection("Task")
            .expect("existing entity target collection should build"),
        )
        .expect("binding should build")
}
