use super::*;

#[test]
fn workspace_probe_existing_delegates_to_routing_intent_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut delegated_workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("intent-admission-probe-delegated")
        .expect("workspace should open");
    let delegated_binding = seeded_probe_binding(request_execution, &mut delegated_workspace);
    let delegated = delegated_workspace
        .probe_existing(
            request_execution,
            delegated_binding,
            test_aspect_touches(["identity.id", "title.value"]),
        )
        .expect("legacy workspace probe should execute");

    let mut canonical_workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("intent-admission-probe-canonical")
        .expect("workspace should open");
    let canonical_binding = seeded_probe_binding(request_execution, &mut canonical_workspace);
    let request = WorthQueryExistingTruthProbeRequest::new(
        canonical_binding,
        test_aspect_touches(["identity.id", "title.value"]),
    )
    .expect("probe request should build");
    let canonical = canonical_workspace
        .probe_existing_intent(request)
        .execute(request_execution)
        .expect("canonical probe intent should execute");

    assert_eq!(delegated, *canonical.probe());
    assert_eq!(
        canonical
            .receipt()
            .execution_provenance()
            .map(|provenance| provenance.entrypoint()),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteExistingTruthProbeRouting)
    );
    assert_eq!(
        canonical
            .receipt()
            .decision_trace_envelope()
            .map(trace_stages),
        Some(vec![
            WorthQueryIntentDecisionTraceStage::RawIntent,
            WorthQueryIntentDecisionTraceStage::Eligibility,
            WorthQueryIntentDecisionTraceStage::AdmittedDecision,
            WorthQueryIntentDecisionTraceStage::ExecutionHandoff,
            WorthQueryIntentDecisionTraceStage::ExecutionOutcome,
        ])
    );
}

#[test]
fn runtime_probe_existing_delegates_to_canonical_routing_intent_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut delegated_workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("intent-admission-probe-runtime-delegated")
        .expect("workspace should open");
    let delegated_binding = seeded_probe_binding(request_execution, &mut delegated_workspace);
    let delegated_request = WorthQueryExistingTruthProbeRequest::new(
        delegated_binding,
        test_aspect_touches(["identity.id", "title.value"]),
    )
    .expect("probe request should build");
    let delegated_runtime = delegated_workspace.into_runtime();
    let delegated = delegated_runtime
        .probe_existing(request_execution, delegated_request)
        .expect("legacy runtime probe should execute");

    let mut canonical_workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("intent-admission-probe-runtime-canonical")
        .expect("workspace should open");
    let canonical_binding = seeded_probe_binding(request_execution, &mut canonical_workspace);
    let canonical_request = WorthQueryExistingTruthProbeRequest::new(
        canonical_binding,
        test_aspect_touches(["identity.id", "title.value"]),
    )
    .expect("probe request should build");
    let canonical_runtime = canonical_workspace.into_runtime();
    let canonical = canonical_runtime
        .probe_existing_intent(canonical_request)
        .execute(request_execution)
        .expect("canonical probe intent should execute");

    assert_eq!(delegated, *canonical.probe());
    assert_eq!(
        canonical.receipt().probe_digest(),
        canonical.probe().probe_digest()
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
