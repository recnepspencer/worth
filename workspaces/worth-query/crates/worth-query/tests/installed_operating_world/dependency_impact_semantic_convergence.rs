use crate::workflow_request::workflow_request;
use worth_query::facade::runtime::ExecutionRequest;
use worth_query::facade::{certification, domain};

use super::installed_operation_fixture::{lineage_workflow_workspace, LineageEvidenceScenario};

#[test]
fn certification_replay_compares_full_lineage_output_and_stage_evidence_semantics() {
    let mut workspace = lineage_workflow_workspace(
        "dependency-impact-lineage-replay",
        domain::WorthQueryOperationLineageContract::Evolve,
        false,
        vec![LineageEvidenceScenario::SingularSuccessor],
    )
    .unwrap();
    let original = super::operation_lineage::bind(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(
            super::operation_lineage::intent(),
            &mut workspace,
            ExecutionRequest::serial(&workflow_request()),
        )
        .unwrap();
    let replay = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        super::operation_lineage::bind(&workspace),
        super::operation_lineage::intent(),
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
        ExecutionRequest::serial(&workflow_request()),
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

#[derive(Clone, Copy)]
enum SourceKind {
    WorkflowRead,
    WorkflowEffect,
    WorkflowLineage,
    WorkflowOutput,
}

fn has_source_kind(
    closure: &domain::WorthQueryCompiledSemanticAspectDependencyClosure,
    expected: SourceKind,
) -> bool {
    closure.dependencies().iter().any(|dependency| {
        matches!(
            (expected, dependency.source()),
            (
                SourceKind::WorkflowRead,
                domain::WorthQuerySemanticAspectDependencyView::RealizedWorkflowRead(_)
            ) | (
                SourceKind::WorkflowEffect,
                domain::WorthQuerySemanticAspectDependencyView::RealizedWorkflowEffect(_)
            ) | (
                SourceKind::WorkflowLineage,
                domain::WorthQuerySemanticAspectDependencyView::RealizedWorkflowLineage(_)
            ) | (
                SourceKind::WorkflowOutput,
                domain::WorthQuerySemanticAspectDependencyView::RealizedWorkflowOutput { .. }
            )
        )
    })
}
