use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;
use worth_relational::facade::authorization::RelationalAuthorizationObservationCounters;

use super::super::application_attempt::authenticated_principal;
use super::super::fixture::{
    installed_capability_world_with_exact_pair_population,
    installed_capability_world_with_same_resource_unrelated,
    installed_delegated_capability_world_at_depth,
    installed_delegated_capability_world_with_unrelated, live_scope, AuthorizationWorld,
};
use super::capability_progression::{admitted_capability_access, time};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DelegationAdmissionWork {
    relational: RelationalAuthorizationObservationCounters,
    signal_dependencies: usize,
    canonical: WorthQueryCanonicalWorkEvidence,
}

#[test]
fn warm_work_has_an_exact_per_link_slope() {
    let root = admission_work(installed_delegated_capability_world_at_depth(0));
    let one_link = admission_work(installed_delegated_capability_world_at_depth(1));
    let two_links = admission_work(installed_delegated_capability_world_at_depth(2));

    assert_eq!(
        root.relational,
        RelationalAuthorizationObservationCounters {
            paths_evaluated: 6,
            adjacency_lists_read: 11,
            adjacency_edges_inspected: 10,
            relation_records_inspected: 12,
            entity_records_inspected: 36,
            predicate_fields_inspected: 19,
            relation_join_index_lookups: 1,
            relation_join_candidates_inspected: 1,
            maximum_frontier_width: 1,
            reconstructive_graph_scans: 0,
            reconstructive_relation_records_scanned: 0,
        }
    );
    assert_eq!(one_link.relational.paths_evaluated, 13);
    assert_eq!(two_links.relational.paths_evaluated, 20);
    assert_same_link_delta(root, one_link, two_links);
    assert_eq!(
        [
            root.relational.maximum_frontier_width,
            one_link.relational.maximum_frontier_width,
            two_links.relational.maximum_frontier_width,
        ],
        [1, 1, 1],
    );
    for work in [root, one_link, two_links] {
        assert_eq!(work.relational.relation_join_index_lookups, 1);
        assert_eq!(work.relational.relation_join_candidates_inspected, 1);
        assert_eq!(work.relational.reconstructive_graph_scans, 0);
        assert_eq!(work.relational.reconstructive_relation_records_scanned, 0);
        assert_eq!(work.canonical, WorthQueryCanonicalWorkEvidence::zero());
    }
}

#[test]
fn complete_unrelated_grants_do_not_enter_warm_admission_work() {
    let baseline = admission_work(installed_delegated_capability_world_with_unrelated(0));
    let populated = admission_work(installed_delegated_capability_world_with_unrelated(256));

    assert_eq!(populated, baseline);
}

#[test]
fn same_resource_unrelated_grants_do_not_enter_exact_grant_selection() {
    let baseline = admission_work(installed_capability_world_with_same_resource_unrelated(0));
    let populated = admission_work(installed_capability_world_with_same_resource_unrelated(256));

    assert_eq!(populated, baseline);
}

#[test]
fn exact_pair_population_overflow_denies_before_authority() {
    let world = installed_capability_world_with_exact_pair_population(65);
    world.authorization_time.script([time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);

    let Err(denial) = admitted_capability_access(&world, &principal, &request, 100) else {
        panic!("an over-budget exact grant population must deny");
    };

    assert_eq!(
        denial.kind(),
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::GrantSelectionLimitExceeded
    );
}

fn admission_work(world: AuthorizationWorld) -> DelegationAdmissionWork {
    world.authorization_time.script([time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let access = admitted_capability_access(&world, &principal, &request, 100).unwrap();
    DelegationAdmissionWork {
        relational: access.relational_counters(),
        signal_dependencies: access.signal_dependency_count(),
        canonical: access.admission_canonical_work(),
    }
}

fn assert_same_link_delta(
    root: DelegationAdmissionWork,
    one_link: DelegationAdmissionWork,
    two_links: DelegationAdmissionWork,
) {
    assert_eq!(
        additive_delta(one_link.relational, root.relational),
        additive_delta(two_links.relational, one_link.relational),
        "root={root:?}, one_link={one_link:?}, two_links={two_links:?}",
    );
    assert_eq!(
        one_link.signal_dependencies - root.signal_dependencies,
        two_links.signal_dependencies - one_link.signal_dependencies,
    );
}

fn additive_delta(
    greater: RelationalAuthorizationObservationCounters,
    lesser: RelationalAuthorizationObservationCounters,
) -> [usize; 10] {
    [
        greater.paths_evaluated - lesser.paths_evaluated,
        greater.adjacency_lists_read - lesser.adjacency_lists_read,
        greater.adjacency_edges_inspected - lesser.adjacency_edges_inspected,
        greater.relation_records_inspected - lesser.relation_records_inspected,
        greater.entity_records_inspected - lesser.entity_records_inspected,
        greater.predicate_fields_inspected - lesser.predicate_fields_inspected,
        greater.relation_join_index_lookups - lesser.relation_join_index_lookups,
        greater.relation_join_candidates_inspected - lesser.relation_join_candidates_inspected,
        greater.reconstructive_graph_scans - lesser.reconstructive_graph_scans,
        greater.reconstructive_relation_records_scanned
            - lesser.reconstructive_relation_records_scanned,
    ]
}
