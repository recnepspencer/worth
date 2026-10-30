pub use super::*;

pub fn assert_declared_mutation_dependencies(
    closure: &domain::WorthQueryCompiledSemanticAspectDependencyClosure,
) {
    assert!(has_source(closure, |source| matches!(
        source,
        domain::WorthQuerySemanticAspectDependencyView::TouchGraphRole("model")
    )));
    assert!(has_source(closure, |source| matches!(
        source,
        domain::WorthQuerySemanticAspectDependencyView::TouchScope("vertex")
    )));
    assert!(has_source(closure, |source| matches!(
        source,
        domain::WorthQuerySemanticAspectDependencyView::WorkflowStageRead {
            graph_read_role: "model"
        }
    )));
    assert!(has_source(closure, |source| matches!(
        source,
        domain::WorthQuerySemanticAspectDependencyView::RealizedWorkflowRead(read)
            if read.role() == "model"
    )));
    assert!(has_source(closure, |source| matches!(
        source,
        domain::WorthQuerySemanticAspectDependencyView::EffectFamily(_)
    )));
    assert!(has_source(closure, |source| matches!(
        source,
        domain::WorthQuerySemanticAspectDependencyView::InstalledInvariant("workflow-invariant:1")
    )));
}

pub fn assert_realized_mutation_dependencies(
    closure: &domain::WorthQueryCompiledSemanticAspectDependencyClosure,
) {
    assert!(has_source(closure, |source| matches!(
        source,
        domain::WorthQuerySemanticAspectDependencyView::RealizedWorkflowEffect(_)
    )));
    assert!(has_source(closure, |source| matches!(
        source,
        domain::WorthQuerySemanticAspectDependencyView::RealizedWorkflowInvariant(_)
    )));
    assert!(has_source(closure, |source| matches!(
        source,
        domain::WorthQuerySemanticAspectDependencyView::RealizedWorkflowOutput { .. }
    )));
}

pub fn assert_mutation_dependency_counters(
    closure: &domain::WorthQueryCompiledSemanticAspectDependencyClosure,
) {
    let counters = closure.counters();
    assert_eq!(counters.effect_contract_edges, 1);
    assert_eq!(counters.invariant_contract_edges, 1);
    assert_eq!(counters.touch_edges, 2);
    assert_eq!(counters.workflow_stage_read_edges, 1);
    assert_eq!(counters.realized_workflow_read_edges, 1);
    assert_eq!(counters.realized_effect_edges, 1);
    assert_eq!(counters.realized_invariant_edges, 1);
    assert_eq!(counters.realized_workflow_output_edges, 1);
}
