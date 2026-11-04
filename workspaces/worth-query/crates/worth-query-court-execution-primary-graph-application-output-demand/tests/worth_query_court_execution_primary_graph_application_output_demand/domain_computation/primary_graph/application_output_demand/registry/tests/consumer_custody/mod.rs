//! A wave's funded interest protects a real row; reclamation grants no proof.
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture;

#[test]
fn a_consumer_handoff_ends_at_real_row_reclamation() {
    let (_lineage, identity) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let registry = WorthQueryOutputDemandRegistry::default();
    let consumer = Arc::new(key("handoff-consumer", 1, 1));
    {
        let mut state = (*registry.state_for_test()).lock().unwrap();
        let (posting, _cancellation) = state.reserve_settlement_vacancy_for_test(&identity).unwrap();
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::settlement_index::SettlementIndex::fill_prepared_for_test(
            &posting,
            &identity,
            Arc::clone(&identity),
            Arc::clone(&consumer),
        );
        let mut row = record(identity.address_for_test().0, worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready(), 0);
        (*row.settlements_mut_for_test()).push(Arc::clone(&identity));

        (*state.records_mut_for_test()).insert_for_test(consumer.as_ref().clone(), row);
    }
    let handoff = registry.retain_consumer_handoff_for_test(&identity).unwrap();
    assert!(handoff.matches(&identity));
    {
        let state = (*registry.state_for_test()).lock().unwrap();
        assert_eq!((*(*state.records_for_test())[consumer.as_ref()].interests_for_test()), 1);
        assert_eq!((*(*state.records_for_test())[consumer.as_ref()].framework_required_count_for_test()), 0);
    }
    // The wave interest protects the posted row while its original owner is alive.
    assert!((*(*registry
        .state_for_test())
        .lock()
        .unwrap()
        .records_for_test())
        .contains_key_for_test(consumer.as_ref()));
    drop(handoff);
    registry.release_product_occurrence_for_test(identity.address_for_test().0);
    assert!(!(*(*registry
        .state_for_test())
        .lock()
        .unwrap()
        .records_for_test())
        .contains_key_for_test(consumer.as_ref()));
    assert!(!registry.posts_settlement_for_test(&identity).unwrap());
    let released = registry.retain_consumer_handoff_for_test(&identity).unwrap();
    assert!(
        !released.matches(&identity),
        "a reclaimed row owns no equality proof"
    );
}
