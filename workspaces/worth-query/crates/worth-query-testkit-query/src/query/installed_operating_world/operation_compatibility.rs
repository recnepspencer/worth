pub mod conditional_boundary_matrix;
pub mod hostile_matrix;
pub mod relationship_laws;

pub use worth_query::facade::{domain, foundation};

pub use super::conditional_node_contract::node;
pub use super::installed_operation_fixture::{
    conditional_controlled_workspace, conditional_workspace, configured_runtime,
    support_dimension_workspace, workspace, GeometryDomain, ReadFamily, ReadVertex,
};

pub fn bind(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
    installed: &domain::WorthQueryInstalledDomainHandle<GeometryDomain>,
) -> domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(installed, ReadVertex)
        .unwrap()
}

pub fn assert_conditional_continuity_work(counters: domain::WorthQueryCompatibilityCounters) {
    assert_eq!(counters.conditional_lowerings_compared, 1);
    assert_eq!(counters.conditional_foundational_comparisons, 31);
    assert_eq!(counters.conditional_liveness_checks, 1);
    assert_eq!(counters.conditional_correspondences_inspected, 1);
    assert_eq!(counters.conditional_targets_inspected, 1);
    assert_eq!(counters.conditional_provider_roles_inspected, 7);
    assert_eq!(counters.conditional_signal_semantic_dimensions_inspected, 8);
    assert_eq!(counters.conditional_signal_affinity_dimensions_inspected, 0);
    assert_eq!(counters.conditional_bridge_affinity_dimensions_inspected, 0);
}

pub fn assert_conditional_affinity_work(counters: domain::WorthQueryCompatibilityCounters) {
    assert_eq!(counters.conditional_lowerings_compared, 1);
    assert_eq!(counters.conditional_foundational_comparisons, 31);
    assert_eq!(counters.conditional_liveness_checks, 2);
    assert_eq!(counters.conditional_correspondences_inspected, 2);
    assert_eq!(counters.conditional_targets_inspected, 2);
    assert_eq!(counters.conditional_provider_roles_inspected, 14);
    assert_eq!(
        counters.conditional_signal_semantic_dimensions_inspected,
        16
    );
    assert_eq!(counters.conditional_signal_affinity_dimensions_inspected, 7);
    assert_eq!(counters.conditional_bridge_affinity_dimensions_inspected, 1);
}

pub fn assert_zero_conditional_owner_work(counters: domain::WorthQueryCompatibilityCounters) {
    assert_eq!(counters.conditional_lowerings_compared, 0);
    assert_eq!(counters.conditional_foundational_comparisons, 0);
    assert_eq!(counters.conditional_liveness_checks, 0);
    assert_eq!(counters.conditional_correspondences_inspected, 0);
    assert_eq!(counters.conditional_targets_inspected, 0);
    assert_eq!(counters.conditional_provider_roles_inspected, 0);
    assert_eq!(counters.conditional_signal_semantic_dimensions_inspected, 0);
    assert_eq!(counters.conditional_signal_affinity_dimensions_inspected, 0);
    assert_eq!(counters.conditional_bridge_affinity_dimensions_inspected, 0);
}
