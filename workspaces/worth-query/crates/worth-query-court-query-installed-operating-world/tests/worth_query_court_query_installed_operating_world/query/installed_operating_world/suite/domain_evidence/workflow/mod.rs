pub(crate) use worth_query_testkit_query::facade::query::suite::domain_evidence::workflow::*;

#[test]
fn workflow_run_ledger_denies_a_locally_valid_counter_regression_atomically() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, probe) =
        evidence_workflow_workspace(request_execution, "domain-evidence-ledger-regression")
            .unwrap();
    probe.set(EvidenceWorkflowMode::LedgerRegression);

    let denial = match bind(&workspace)
        .admit_workflow_resources(
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(evidence_workflow_intent(), &mut workspace)
    {
        TransitionOutcome::Denied(domain::WorthQueryWorkflowReexecutionStop::Advance(denial)) => {
            denial
        }
        _ => panic!("the operation-scoped counter regression did not deny workflow advancement"),
    };
    assert_eq!(
        denial.kind(),
        &domain::WorthQueryWorkflowAdvanceDenialKind::DomainEvidence(
            domain::WorthQueryDomainEvidenceAdmissionDenialKind::LedgerRegression
        )
    );
    assert_eq!(denial.completed_stage_receipts().len(), 2);
    let start = stage_evidence(denial.completed_stage_receipts(), "start");
    assert_eq!(counter(start, "candidate-comparisons").initial(), 0);
    assert_eq!(counter(start, "candidate-comparisons").observed(), 6);
    assert!(denial
        .completed_stage_receipts()
        .iter()
        .all(|receipt| receipt.stage_identity() != "left"));
}

#[test]
fn claimed_output_mismatch_denies_before_a_stage_receipt_exists() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, probe) = evidence_workflow_workspace(
        request_execution,
        "domain-evidence-workflow-output-mismatch",
    )
    .unwrap();
    probe.set(EvidenceWorkflowMode::OutputOccurrenceMismatch);

    let denial = match bind(&workspace)
        .admit_workflow_resources(
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(evidence_workflow_intent(), &mut workspace)
    {
        TransitionOutcome::Denied(domain::WorthQueryWorkflowReexecutionStop::Advance(denial)) => {
            denial
        }
        TransitionOutcome::Success(_) => panic!("mismatched output produced a stage receipt"),
        _ => panic!("mismatched output did not deny exact stage completion"),
    };
    assert_eq!(
        denial.kind(),
        &domain::WorthQueryWorkflowAdvanceDenialKind::DomainEvidence(
            domain::WorthQueryDomainEvidenceAdmissionDenialKind::TransformationSummaryMismatch,
        )
    );
    assert!(denial.completed_stage_receipts().is_empty());
    assert!(denial.graph_receipts().is_empty());
}
