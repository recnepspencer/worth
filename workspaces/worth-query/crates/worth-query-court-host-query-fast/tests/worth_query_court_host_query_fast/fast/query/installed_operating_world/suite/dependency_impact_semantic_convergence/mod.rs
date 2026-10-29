pub(crate) use worth_query_worlds::facade::query::suite::dependency_impact_semantic_convergence::*;

#[test]
fn certification_replay_compares_full_lineage_output_and_stage_evidence_semantics() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = lineage_workflow_workspace(
        request_execution,
        "dependency-impact-lineage-replay",
        domain::WorthQueryOperationLineageContract::Evolve,
        false,
        vec![LineageEvidenceScenario::SingularSuccessor],
    )
    .unwrap();
    let original = super::operation_lineage::bind(&workspace)
        .admit_workflow_resources(
            worth_query_worlds::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(super::operation_lineage::intent(), &mut workspace)
        .unwrap();
    let replay = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        super::operation_lineage::bind(&workspace),
        super::operation_lineage::intent(),
        worth_query_worlds::facade::query::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
    )
    .unwrap();
    let original_closure = original.semantic_aspect_dependency_closure().unwrap();
    assert_eq!(
        replay.comparison(),
        &domain::WorthQueryReplayComparison::Equivalent
    );
    for source_kind in [
        SourceKind::WorkflowRead,
        SourceKind::WorkflowEffect,
        SourceKind::WorkflowLineage,
        SourceKind::WorkflowOutput,
    ] {
        assert!(has_source_kind(original_closure, source_kind));
    }
    assert!(replay
        .replay_semantics()
        .stages()
        .iter()
        .any(|stage| !stage.effects().is_empty() && !stage.lineage().is_empty()));
}
