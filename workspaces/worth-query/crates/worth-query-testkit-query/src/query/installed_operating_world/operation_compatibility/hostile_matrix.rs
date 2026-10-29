pub mod fixture;

pub use worth_foundational::facade::{CanonicalBasisLocus, InternedString};
pub use worth_query::facade::{domain, foundation};

pub use super::super::installed_operation_fixture::{
    configured_runtime, support_dimension_workspace, GeometryDomain, ReadFamily, ReadVertex,
};
pub use fixture::{bind_branch, bind_current, no_primary_read_runtime};
pub type BoundVertex = domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;
pub const PORTABLE_OPERATION_COMPARISON_DIMENSIONS: usize = 45;

pub fn current_relationship_counters(
    subject: &BoundVertex,
    candidate: &BoundVertex,
) -> [domain::WorthQueryCompatibilityCounters; 4] {
    [
        subject
            .same_installation_with(candidate)
            .unwrap()
            .counters(),
        subject.replacement_with(candidate).unwrap().counters(),
        subject.compatible_basis_with(candidate).unwrap().counters(),
        subject
            .execution_sharing_with(candidate)
            .unwrap()
            .counters(),
    ]
}

pub fn assert_current_success_costs(counters: [domain::WorthQueryCompatibilityCounters; 4]) {
    assert_eq!(
        counters.map(|counters| counters.retained_authority_checks),
        [10, 12, 10, 17]
    );
    assert_eq!(counters[0].portable_contract_comparisons, 0);
    assert_eq!(counters[0].portable_variable_items_submitted, 0);
    assert_eq!(counters[0].canonical_comparisons, 0);
    for counters in &counters[1..] {
        assert_eq!(
            counters.portable_contract_comparisons,
            PORTABLE_OPERATION_COMPARISON_DIMENSIONS
        );
        assert_eq!(counters.canonical_comparisons, 5);
        assert!(counters.portable_variable_items_submitted > 0);
        assert_eq!(counters.portable_conditional_nodes_submitted, 0);
        assert_eq!(counters.conditional_lowerings_compared, 0);
        assert_zero_forbidden_work(*counters);
    }
}

pub fn assert_zero_forbidden_work(counters: domain::WorthQueryCompatibilityCounters) {
    assert_eq!(counters.lower_runtime_contacts, 0);
    assert_eq!(counters.execution_calls, 0);
    assert_eq!(counters.maintenance_calls, 0);
}

pub fn bind_read_vertex(
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
