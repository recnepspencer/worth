//! Refusing discovery cannot classify a displaced settlement as unposted.
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::facade::domain_computation::WorthQueryInvalidationResources;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner;

#[test]
fn registry_post_commit_discovery_releases_an_unposted_displaced() {
    let (_lineage, displaced) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let registry = WorthQueryOutputDemandRegistry::default();
    let selected = Arc::new(key("retirement-result", 2, 1));

    {
        let mut state = (*registry.state_for_test()).lock().unwrap();
        state.records.insert(
            selected.as_ref().clone(),
            record(occurrence(), DemandState::Running, 1),
        );
    }
    let resources = WorthQueryInvalidationResources::new();
    let owner = SourceInvalidationOwner::new(resources, 1);

    let pins = Arc::strong_count(&displaced);
    // This invokes the same completion entry as both publication callers. The real
    // recorded identity has no demand posting; the owner must retain its
    // release when this tiny later-maintenance admission cannot retire it.
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::settlement_retirement::SupersededSettlements::new_for_test(registry.clone(), selected)
        .retire(Some(Arc::clone(&displaced)), &owner);
    assert_eq!(
        Arc::strong_count(&displaced),
        pins,
        "an unposted displaced settlement releases its original holder"
    );
    let retired = owner.retire_released(std::iter::empty());
    assert!(
        retired.is_empty(),
        "completed retirement leaves no synthetic retry holder"
    );
    drop(retired);
    assert_eq!(Arc::strong_count(&displaced), pins);
    assert!(owner.retire_released(std::iter::empty()).is_empty());
}
