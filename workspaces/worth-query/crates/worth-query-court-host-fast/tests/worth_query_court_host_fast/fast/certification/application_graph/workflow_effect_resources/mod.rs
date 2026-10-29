pub(crate) use worth_query_worlds::facade::certification::application_graph::workflow_effect_resources::*;

#[test]
fn approval_publishes_all_review_evidence_without_resource_estimates() {
    let key = 53_000;
    let (application, instance, proposal, required) = approval_ready(6, key);
    let result = approve_instance(
        &application,
        instance,
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        key + 100,
    );
    assert!(
        matches!(result, Ok(WorkflowProgressOutcome::Completed(_))),
        "the complete authored review graph must approve: {result:?}"
    );
}
