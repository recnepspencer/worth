use super::*;

#[test]
fn execute_read_family_delegates_to_canonical_admission_and_execution_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = read_runtime(request_execution);
    let mut delegated_workspace =
        WorthQueryWorkspace::new("delegated-read-family", runtime).expect("workspace should build");
    let family = identity_read_family(&mut delegated_workspace, "tasks");
    let delegated = delegated_workspace
        .execute_read_family(request_execution, &family)
        .expect("delegated read family should execute");

    let runtime = read_runtime(request_execution);
    let mut canonical_workspace =
        WorthQueryWorkspace::new("canonical-read-family", runtime).expect("workspace should build");
    let canonical_family = identity_read_family(&mut canonical_workspace, "tasks");
    let review = canonical_workspace
        .review_read_execution(canonical_family, None)
        .expect("canonical read review should succeed");
    let handoff = canonical_workspace
        .resolve_reviewed_admitted_read_execution_handoff(review)
        .expect("canonical read handoff should admit");
    let binding = canonical_workspace
        .into_runtime_read_execution_binding(handoff)
        .expect("canonical read binding should prepare");
    let canonical = canonical_workspace
        .execute_bound_read_execution(request_execution, binding)
        .expect("canonical read binding should execute");

    assert_eq!(delegated.rows(), canonical.rows());
    assert_eq!(
        delegated.receipt().query_digest(),
        canonical.receipt().query_digest()
    );
    assert_eq!(
        delegated.receipt().result_digest(),
        canonical.receipt().result_digest()
    );
    assert_eq!(
        delegated.receipt().execution_provenance_chain_digest(),
        canonical.receipt().execution_provenance_chain_digest()
    );
    assert_eq!(
        delegated
            .receipt()
            .decision_trace_envelope()
            .map(WorthQueryIntentDecisionTraceEnvelope::trace_digest),
        canonical
            .receipt()
            .decision_trace_envelope()
            .map(WorthQueryIntentDecisionTraceEnvelope::trace_digest)
    );
}

#[test]
fn workspace_read_delegates_to_live_read_intent_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = read_runtime(request_execution);
    let mut delegated_workspace =
        WorthQueryWorkspace::new("delegated-live-read", runtime).expect("workspace should build");
    let delegated_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = delegated_workspace
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
    let delegated = delegated_workspace.read(request_execution, &delegated_view);

    let runtime = read_runtime(request_execution);
    let mut canonical_workspace =
        WorthQueryWorkspace::new("canonical-live-read", runtime).expect("workspace should build");
    let canonical_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = canonical_workspace
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
    let canonical = canonical_workspace
        .read_live_intent(&canonical_view)
        .execute(request_execution)
        .expect("live read common path should execute");

    assert_eq!(delegated, canonical.rows());
    assert_eq!(canonical.receipt().view_name(), "tasks.table");
    assert_eq!(
        canonical
            .receipt()
            .execution_provenance()
            .map(|provenance| provenance.entrypoint()),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteLiveRead)
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
fn runtime_read_live_delegates_to_canonical_live_read_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = read_runtime(request_execution);
    let live_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");

    let delegated = runtime.read_live(request_execution, &live_view);
    let canonical = runtime
        .read_live_result(request_execution, &live_view)
        .expect("canonical runtime live read should execute");

    assert_eq!(delegated, canonical.rows());
    assert_eq!(canonical.receipt().view_name(), "tasks.table");
    assert_eq!(
        canonical
            .receipt()
            .execution_provenance()
            .map(|provenance| provenance.entrypoint()),
        Some(WorthQueryIntentAdmissionCoveredEntrypoint::ExecuteLiveRead)
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
