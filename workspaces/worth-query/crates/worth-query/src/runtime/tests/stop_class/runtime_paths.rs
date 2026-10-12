use super::super::support::*;

#[test]
fn intent_execution_routing_stop_class_preserves_stage_evidence_and_source() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = bridge_runtime_with_support_and_intent_authority(
        request_execution,
        intent_support_profile(),
        TestIntentAuthority,
    );
    let declaration = WorthQueryIntentDeclaration::strategy_commit(
        "phase-three-routing-stop-class",
        "strategy.intent.reconcile",
        "1.0",
        "intent.reconcile.input.v1",
        test_intent_input([("entity", "task-1")]),
    );
    let handoff = runtime
        .admit_authoritative_intent_for_execution(declaration.clone())
        .expect("authoritative handoff should admit");
    let binding = runtime.prepare_authoritative_intent_execution_binding(handoff.clone());
    let execution = runtime
        .backend
        .execute_intent(request_execution, binding.declaration())
        .expect("backend execution should succeed");
    let admitted_handoff = WorthQueryAdmittedIntentExecutionHandoff::from(handoff);
    let snapshot_evidence_identity = execution
        .mutation_receipt()
        .snapshot_identity
        .evidence_identity();
    let execution_provenance =
        WorthQueryIntentExecutionProvenance::for_shared_execution_typed_parts(
            binding.family(),
            binding.entrypoint(),
            binding.execution_seam(),
            binding.handoff().decision_digest(),
            binding.handoff().handoff_digest(),
            binding.binding_digest(),
            execution.outcome_digest(),
            &snapshot_evidence_identity,
        );
    let decision_trace_envelope = WorthQueryIntentDecisionTraceEnvelope::for_admitted_execution(
        &admitted_handoff,
        &execution,
    );
    let error = runtime.intent_execution_routing_error(
        &declaration,
        &execution,
        execution_provenance.clone(),
        decision_trace_envelope.clone(),
        WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name: "tasks.phase-three-routing-stop-class".to_string(),
            stage: "delivery-window",
            message: "simulated route failure".to_string(),
        },
    );

    match error.stop_class() {
        WorthQueryStopClass::IntentExecutionRoutingFailed {
            intent_name,
            stage,
            evidence,
            source,
            ..
        } => {
            assert_eq!(intent_name, "phase-three-routing-stop-class");
            assert_eq!(stage, "post-execution-routing");
            assert_eq!(
                evidence
                    .execution_provenance()
                    .execution_provenance_chain_digest(),
                execution_provenance.execution_provenance_chain_digest()
            );
            assert_eq!(
                evidence.decision_trace_envelope().trace_digest(),
                decision_trace_envelope.trace_digest()
            );
            match source.stop_class() {
                WorthQueryStopClass::RuntimeDeclarationFailed {
                    kind,
                    name,
                    stage,
                    message,
                } => {
                    assert_eq!(
                        kind,
                        WorthQueryRuntimeDeclarationFailureKind::LiveSubscriptionInstallation
                    );
                    assert_eq!(name, "tasks.phase-three-routing-stop-class");
                    assert_eq!(stage, "delivery-window");
                    assert_eq!(message, "simulated route failure");
                }
                other => panic!("expected routed source stop class, got {other:?}"),
            }
        }
        other => panic!("expected intent execution routing stop class, got {other:?}"),
    }
}
