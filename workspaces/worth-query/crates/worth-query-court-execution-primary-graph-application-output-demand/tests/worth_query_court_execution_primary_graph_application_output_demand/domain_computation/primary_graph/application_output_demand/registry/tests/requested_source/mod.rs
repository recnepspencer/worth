//! A Ready and real membership cannot substitute for an admitted source.

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::required_work::RequiredWorkWorthQueryOutputDemandRegistryForTest;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

#[test]
fn a_cached_ready_without_its_source_cannot_claim_requested_execution() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let (interest, _) = registry.fixture_admitted_work_membership(occurrence());
    let key = (*interest.key_for_test()).clone();
    (*(*(*registry
        .state_for_test())
        .lock()
        .unwrap()
        .records_mut_for_test())
        .get_mut(&key)
        .unwrap()
        .state_mut_for_test()) = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready();
    drop(interest);

    assert!(registry
        .claim_cached_requested_ready_for_test(&key)
        .unwrap()
        .is_none());
    let state = (*registry.state_for_test()).lock().unwrap();
    assert!((*state.records_for_test())[&key].has_cached_ready_for_test());
    assert!(!(*state.records_for_test())[&key].is_required_for_test());
    assert!((*(*state.records_for_test())[&key].readmission_source_for_test()).is_none());
    assert!(!(*state.required_keys_for_test()).contains(&key));
}
