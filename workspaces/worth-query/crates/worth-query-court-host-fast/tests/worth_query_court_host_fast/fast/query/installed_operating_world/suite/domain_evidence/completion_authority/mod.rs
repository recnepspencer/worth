pub(crate) use worth_query_worlds::facade::query::suite::domain_evidence::completion_authority::*;

#[test]
fn exact_direct_completions_bind_zero_graph_output_snapshot_and_resource_attempt() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = evidence_workspace(
        request_execution,
        "domain-evidence-completion-twins",
        EvidenceScenario::Honest,
        domain::WorthQueryArtifactRedactionPosture::NotRequired,
    )
    .unwrap();
    let first_admitted = admit_exact_completion(&workspace);
    let second_admitted = admit_exact_completion(&workspace);
    let first = execute_exact_completion(first_admitted, &mut workspace, 0);
    let second = execute_exact_completion(second_admitted, &mut workspace, 0);

    assert_eq!(
        first.output, second.output,
        "the twin must hold output constant"
    );
    assert_eq!(first.operation, second.operation);
    assert_eq!(first.binding, second.binding);
    assert_eq!(first.basis, second.basis);
    assert_eq!(first.snapshot, second.snapshot);
    assert_ne!(first.provider_session, second.provider_session);
    assert_ne!(first.provider_attempt, second.provider_attempt);
    assert_ne!(first.execution_occurrence, second.execution_occurrence);
}

#[test]
fn exact_workflow_completions_bind_stage_run_snapshot_and_resource_attempt() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, _) = evidence_workflow_workspace(
        request_execution,
        "domain-evidence-workflow-completion-twins",
    )
    .unwrap();
    let first = execute_workflow_completion(&mut workspace, 0);
    let second = execute_workflow_completion(&mut workspace, 0);

    assert_eq!(
        first.output, second.output,
        "the stage output must remain constant"
    );
    assert_eq!(first.operation, second.operation);
    assert_eq!(first.binding, second.binding);
    assert_eq!(first.basis, second.basis);
    assert_eq!(first.stage, second.stage);
    assert_eq!(first.snapshot, second.snapshot);
    assert_ne!(first.run, second.run);
    assert_ne!(first.provider_session, second.provider_session);
    assert_ne!(first.provider_attempt, second.provider_attempt);
    assert_ne!(first.execution_occurrence, second.execution_occurrence);
}

#[test]
fn exact_direct_completions_bind_ordered_nonempty_graph_receipts() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace =
        evidence_graph_workspace(request_execution, "domain-evidence-completion-graph-twins")
            .unwrap();
    let first_admitted = admit_exact_completion(&workspace);
    let second_admitted = admit_exact_completion(&workspace);
    let first = execute_exact_completion(first_admitted, &mut workspace, 1);
    let second = execute_exact_completion(second_admitted, &mut workspace, 1);

    assert_equal_direct_semantics(&first, &second);
    assert_eq!(first.graph_receipts.len(), 1);
    assert_eq!(second.graph_receipts.len(), 1);
    assert_ne!(first.graph_receipts, second.graph_receipts);
    assert_ne!(first.execution_occurrence, second.execution_occurrence);
}

#[test]
fn exact_workflow_completions_bind_ordered_nonempty_graph_receipts() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, _) = evidence_graph_workflow_workspace(
        request_execution,
        "domain-evidence-workflow-graph-twins",
    )
    .unwrap();
    let first = execute_workflow_completion(&mut workspace, 1);
    let second = execute_workflow_completion(&mut workspace, 1);

    assert_equal_workflow_semantics(&first, &second);
    assert_eq!(first.graph_receipts.len(), 1);
    assert_eq!(second.graph_receipts.len(), 1);
    assert_ne!(first.graph_receipts, second.graph_receipts);
    assert_ne!(first.execution_occurrence, second.execution_occurrence);
}
