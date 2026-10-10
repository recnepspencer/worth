use super::representative::worth_query_domain_capability_representative_report;

#[test]
fn representative_report_emits_all_required_foundation_digests() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let report = worth_query_domain_capability_representative_report(request_execution);

    for key in [
        "query_digest",
        "intent_declaration_digest",
        "domain_capability_contribution_request_digest",
        "admission_artifact_digest",
        "support_artifact_digest",
        "workflow_artifact_digest",
        "continuity_artifact_digest",
        "aftermath_artifact_digest",
        "explanation_artifact_digest",
        "capability_support_row_digest",
        "domain_invariant_denial_digest",
        "decision_trace_digest",
        "support_traceability_digest",
        "failure_digest",
    ] {
        assert!(report
            .digest_for(key)
            .is_some_and(|digest| !digest.is_empty()));
    }
    assert!(report.trace_width() > 0);
    assert!(report.support_width() >= 2);
}
