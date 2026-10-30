pub use worth_query::facade::{certification, domain};

pub use super::installed_operation_fixture::{lineage_workflow_workspace, LineageEvidenceScenario};

#[derive(Clone, Copy)]
pub enum SourceKind {
    WorkflowRead,
    WorkflowEffect,
    WorkflowLineage,
    WorkflowOutput,
}

pub fn has_source_kind(
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
