//! Ordinary and request-admitted planning preserve the same actual Native requirement.
use super::*;
use worth_query_admission::facade::{application_query::*, graph_read_access::*};
use worth_query_admission::integration::{
    derive_graph_read_access_requirements_for_contract,
    derive_graph_read_access_requirements_for_contract_admitted,
};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_installation::facade::WorthQueryReadGraphPlanningContract;
#[test]
fn installed_orbit_requires_complete_traversal_and_native_aggregate_support() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = world();
    let query = read::installed(&world, Orbit::reference());
    let relation = WorthQueryReadGraphPlanningContract::relation(query.read_graph(), 0).unwrap();
    assert!(relation.complete_closure && relation.aggregate_selection);
    let parameters =
        admit_application_query_parameters(&query, ApplicationQueryParameterSet::new()).unwrap();
    let budget =
        worth_foundational::facade::CanonicalDigestWorkBudget::new(4096, 1024 * 1024).unwrap();
    let ordinary = derive_graph_read_access_requirements_for_contract(
        query.read_graph(),
        WorthQueryApplicationQueryLane::OneShot,
        32,
        parameters.identity(),
        budget,
    )
    .unwrap();
    let admitted = derive_graph_read_access_requirements_for_contract_admitted(
        query.read_graph(),
        WorthQueryApplicationQueryLane::OneShot,
        32,
        parameters.identity(),
        budget,
        &mut |_, _| Ok::<_, ()>(()),
    )
    .unwrap();
    assert_eq!(ordinary, admitted);
    assert_eq!(ordinary.counters().aggregate_selection_count(), 1);
    assert_eq!(ordinary.counters().predicate_support_count(), 0);
    assert_eq!(ordinary.counters().ordering_support_count(), 0);
    assert_eq!(ordinary.counters().traversal_workset_count(), 1);
    assert_eq!(ordinary.counters().visited_set_count(), 1);

    let selected = ordinary
        .rows()
        .iter()
        .filter(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::AggregateSelection)
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].traversal_operator(),
        Some(&WorthQueryGraphReadTraversalOperator::AggregateSelection)
    );
    assert_eq!(
        selected[0].complexity_contract(),
        &WorthQueryGraphReadAccessComplexityContract::CompleteTraversalWorkset
    );
    assert_eq!(
        selected[0].memory_estimate_basis(),
        &WorthQueryGraphReadAccessMemoryEstimateBasis::DiscoveredWorkset
    );
    let inventory = WorthQueryGraphIndexInventory::from_current_runtime_support();
    let matches = match_graph_index_inventory_for_requirements(&ordinary, &inventory);
    assert!(
        matches
            .matches()
            .iter()
            .all(|row| row.outcome().as_str() == "exact_match"),
        "{matches:?}"
    );
    let native = matches
        .matches()
        .iter()
        .find(|row| {
            row.requirement_kind() == &WorthQueryGraphReadAccessRequirementKind::AggregateSelection
        })
        .unwrap();
    assert_eq!(native.lifecycle_owner().as_str(), "lower_runtime");
    let missing = WorthQueryGraphIndexInventory::from_rows(
        inventory
            .rows()
            .iter()
            .filter(|row| {
                row.requirement_kind()
                    != &WorthQueryGraphReadAccessRequirementKind::AggregateSelection
            })
            .cloned()
            .collect(),
    );
    assert!(
        match_graph_index_inventory_for_requirements(&ordinary, &missing)
            .matches()
            .iter()
            .any(|row| row.outcome().as_str() != "exact_match")
    );
}
