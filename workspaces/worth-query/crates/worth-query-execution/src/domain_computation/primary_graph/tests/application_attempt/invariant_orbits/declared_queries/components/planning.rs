//! Components require Native selection without inventing a coupling relation view.
use super::*;
use worth_query_admission::facade::{application_query::*, graph_read_access::*};
use worth_query_admission::integration::{
    derive_graph_read_access_requirements_for_contract,
    derive_graph_read_access_requirements_for_contract_admitted,
};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_installation::facade::WorthQueryReadGraphPlanningContract;
#[test]
fn installed_components_require_native_support_in_both_planning_lanes() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = world();
    let query = read::installed(&world, Census::reference());
    assert!(WorthQueryReadGraphPlanningContract::component_selection(query.read_graph()).is_some());
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
    let layout = world.application.runtime.primary_graph().unwrap().layout();
    let actual=crate::domain_computation::primary_graph::application_query::primary_graph_support_inventory(query.read_graph(), layout, None, None, &ordinary).unwrap();
    assert!(
        match_graph_index_inventory_for_requirements(&ordinary, &actual)
            .matches()
            .iter()
            .all(|row| row.outcome().as_str() == "exact_match")
    );
    let noncomponent = read::installed(&world, Orbit::reference());
    let wrong=crate::domain_computation::primary_graph::application_query::primary_graph_support_inventory(noncomponent.read_graph(), layout, None, None, &ordinary).unwrap();
    assert!(wrong.row_for_requirement_kind(&WorthQueryGraphReadAccessRequirementKind::AggregateSelection).is_none(), "a relationless aggregate row cannot borrow orbit support or omit its installed component contract");

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
    assert!(selected[0].relation_name().is_none());
    assert!(selected[0].relation_authority().is_none());
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
