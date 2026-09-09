use worth_query::facade::{certification, domain, foundation, runtime};

use super::installed_operation_fixture::{
    mutation_workflow_workspace, workflow_workspace, GeometryDomain, MutationFamily, ReadFamily,
    WorkflowMutation, WorkflowRead,
};
use super::operation_reexecution::intent;

mod workflow_closure;

fn has_source(
    closure: &domain::WorthQueryCompiledSemanticAspectDependencyClosure,
    predicate: impl Fn(domain::WorthQuerySemanticAspectDependencyView<'_>) -> bool,
) -> bool {
    closure
        .dependencies()
        .iter()
        .any(|dependency| predicate(dependency.source()))
}

fn assert_exact_d_work(
    closure: &domain::WorthQueryCompiledSemanticAspectDependencyClosure,
    d: usize,
) {
    let counters = closure.counters();
    assert_eq!(counters.compiled_dependency_count, d);
    assert_eq!(counters.canonical_traversal_edges, d);
    assert_eq!(counters.uniqueness_hash_checks, d);
    assert_eq!(counters.closure_edges_traversed, d - 1);
    assert_eq!(
        closure.measured_compilation_width(),
        counters.compiled_dependency_count
            + counters.impact_index_dependency_visits
            + counters.impact_index_entries
            + counters.impact_mask_propagation_edges
            + counters.workflow_graph_edges_traversed
    );
    assert_eq!(closure.closure_evidence().dependency_count(), d);
    assert_eq!(closure.closure_evidence().closure_edge_count(), d - 1);
    assert_eq!(counters.unrelated_definition_scans, 0);
    assert_eq!(counters.unrelated_runtime_scans, 0);
    assert_eq!(counters.consumer_registry_scans, 0);
}

fn bind_workflow(
    workspace: &runtime::WorthQueryWorkspace,
) -> domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(GeometryDomain).unwrap();
    workspace
        .observe_operating_world()
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap()
}
