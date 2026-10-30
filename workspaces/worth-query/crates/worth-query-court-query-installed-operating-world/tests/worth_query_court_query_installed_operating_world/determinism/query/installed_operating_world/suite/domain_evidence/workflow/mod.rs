pub(crate) use worth_query_testkit_query::facade::query::suite::domain_evidence::workflow::*;

#[test]
fn replay_compares_mandatory_core_and_ignores_policy_omitted_sidecars() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, probe) =
        evidence_workflow_workspace(request_execution, "domain-evidence-replay-sidecars").unwrap();
    let original = bind(&workspace)
        .admit_workflow_resources(
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(evidence_workflow_intent(), &mut workspace)
        .unwrap();
    probe.set(EvidenceWorkflowMode::OmitSidecars);
    let candidate = bind(&workspace)
        .admit_workflow_resources(
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(evidence_workflow_intent(), &mut workspace)
        .unwrap();

    for stage in ["start", "left"] {
        let original_evidence = stage_evidence(original.stage_receipts(), stage);
        let candidate_evidence = stage_evidence(candidate.stage_receipts(), stage);
        assert_eq!(original_evidence.core(), candidate_evidence.core());
        assert!(original_evidence.counter_sidecar().records().is_some());
        assert!(original_evidence.decision_sidecar().records().is_some());
        assert!(original_evidence.candidate_sidecar().records().is_some());
        assert!(original_evidence
            .transformation_sidecar()
            .records()
            .is_some());
        assert!(matches!(
            candidate_evidence.counter_sidecar(),
            domain::WorthQueryAdmittedDomainEvidenceSidecar::Omitted
        ));
        assert!(matches!(
            candidate_evidence.decision_sidecar(),
            domain::WorthQueryAdmittedDomainEvidenceSidecar::Omitted
        ));
        assert!(matches!(
            candidate_evidence.candidate_sidecar(),
            domain::WorthQueryAdmittedDomainEvidenceSidecar::Omitted
        ));
        assert!(matches!(
            candidate_evidence.transformation_sidecar(),
            domain::WorthQueryAdmittedDomainEvidenceSidecar::Omitted
        ));
    }
    assert_eq!(
        domain::compare_exact_workflow_traces(
            &original.semantics(),
            &candidate.semantics(),
            Default::default(),
        ),
        domain::WorthQueryReplayComparison::Equivalent
    );

    let replay = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        bind(&workspace),
        evidence_workflow_intent(),
        worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
    )
    .unwrap();
    assert_eq!(
        replay.comparison(),
        &domain::WorthQueryReplayComparison::Equivalent
    );
    assert_eq!(replay.counters().semantic_stage_comparisons, 4);
}

#[test]
fn certification_replay_cannot_waive_exact_mandatory_core_drift() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, probe) =
        evidence_workflow_workspace(request_execution, "domain-evidence-replay-core-drift")
            .unwrap();
    let original = bind(&workspace)
        .admit_workflow_resources(
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(evidence_workflow_intent(), &mut workspace)
        .unwrap();
    probe.set(EvidenceWorkflowMode::ReplayCoreDrift);

    let replay = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        bind(&workspace),
        evidence_workflow_intent(),
        worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
    )
    .unwrap();
    assert_eq!(
        replay.comparison(),
        &domain::WorthQueryReplayComparison::Diverged(
            domain::WorthQueryReplayDivergence::DomainEvidence {
                stage: "left".into()
            }
        )
    );
    assert_eq!(replay.counters().semantic_stage_comparisons, 1);
}
