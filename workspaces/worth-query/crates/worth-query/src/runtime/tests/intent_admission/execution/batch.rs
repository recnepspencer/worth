use super::*;

#[test]
fn batch_write_delegates_to_canonical_admission_and_execution_handoff() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let commands = vec![
        WorthQueryAspectMutationBuilder::new()
            .set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-batch-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("batch title one"),
            )
            .build_insert("Task")
            .expect("batch command should build"),
        WorthQueryAspectMutationBuilder::new()
            .set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-batch-2"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("batch title two"),
            )
            .build_insert("Task")
            .expect("batch command should build"),
    ];

    let mut delegated_runtime =
        intent_runtime_with_authority(request_execution, TestIntentAuthority);
    let delegated = delegated_runtime
        .write_batch(request_execution, commands.clone())
        .expect("delegated batch write should execute");

    let mut canonical_runtime =
        intent_runtime_with_authority(request_execution, TestIntentAuthority);
    let review = canonical_runtime
        .review_authoritative_runtime_write_batch(commands)
        .expect("canonical batch write review should succeed");
    let handoff = canonical_runtime
        .resolve_reviewed_admitted_authoritative_write_batch_handoff(review)
        .expect("canonical batch handoff should admit");
    let binding = canonical_runtime.prepare_authoritative_mutation_batch_execution_binding(handoff);
    let canonical = canonical_runtime
        .execute_authoritative_mutation_batch_execution_binding(request_execution, binding)
        .expect("canonical batch binding should execute");

    assert_eq!(
        delegated
            .execution_provenance()
            .map(|p| p.execution_provenance_chain_digest()),
        canonical
            .execution_provenance()
            .map(|p| p.execution_provenance_chain_digest())
    );
    assert_eq!(
        delegated
            .decision_trace_envelope()
            .map(WorthQueryIntentDecisionTraceEnvelope::trace_digest),
        canonical
            .decision_trace_envelope()
            .map(WorthQueryIntentDecisionTraceEnvelope::trace_digest)
    );
    assert_eq!(delegated.write_count(), 2);
}
