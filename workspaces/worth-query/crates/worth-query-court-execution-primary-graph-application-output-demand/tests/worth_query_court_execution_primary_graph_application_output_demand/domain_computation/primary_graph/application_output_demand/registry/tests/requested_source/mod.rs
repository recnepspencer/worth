//! A Ready and real membership cannot substitute for an admitted source.

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::required_work::RequiredWorkWorthQueryOutputDemandRegistryForTest;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

#[test]
fn a_cached_ready_without_its_source_cannot_claim_requested_execution() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let (interest, _) = registry.fixture_admitted_work_membership(occurrence());
    let key = (*interest.key_for_test()).clone();
    (*registry
        .state_for_test())
        .lock()
        .unwrap()
        .records
        .get_mut(&key)
        .unwrap()
        .state = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready();
    drop(interest);

    assert!(registry
        .claim_cached_requested_ready_for_test(&key)
        .unwrap()
        .is_none());
    let state = (*registry.state_for_test()).lock().unwrap();
    assert!(state.records[&key].has_cached_ready());
    assert!(!state.records[&key].is_required());
    assert!(state.records[&key].readmission_source.is_none());
    assert!(!state.required_keys.contains(&key));
}
