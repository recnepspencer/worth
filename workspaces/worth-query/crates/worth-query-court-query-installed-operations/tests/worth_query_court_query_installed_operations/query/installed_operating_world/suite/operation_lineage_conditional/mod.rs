pub(crate) use worth_query_testkit_query::facade::query::suite::operation_lineage_conditional::*;

#[test]
fn conditional_no_change_or_deferral_cannot_claim_fresh_lineage() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = reverted_conditional_lineage_workflow_workspace(
        request_execution,
        "conditional-lineage-reverted",
        stage_node(),
    )
    .unwrap();

    assert!(matches!(
        bind(&workspace).admit_workflow_resources(worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(), &workspace).unwrap().reexecute(intent(), &mut workspace),
        TransitionOutcome::Deferred(
            domain::WorthQueryWorkflowReexecutionStop::ConditionalDeferred { stage_identity, .. }
        ) if stage_identity == "publish"
    ));
}
