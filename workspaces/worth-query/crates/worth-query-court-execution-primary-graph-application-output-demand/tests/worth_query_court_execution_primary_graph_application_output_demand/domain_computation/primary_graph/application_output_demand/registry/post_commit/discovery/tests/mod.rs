use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
    use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence;
    use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::record;
    use std::sync::Arc;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::settlement_retirement::SupersededSettlements;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::post_commit::discovery::after_publication;
#[test]
fn registry_completed_discovery_body_keeps_its_occurrence_bound() {
    let receipts = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::receipt_batch();
    let mut contacts = Vec::new();
    for (neighbors, entries) in [(1, 1), (64, 4)] {
        let registry = WorthQueryOutputDemandRegistry::default();
        let selected = key("posted-discovery", 3, 1);
        let mut state = (*registry.state_for_test()).lock().unwrap();
        for generation in 1..=3 {
            state.records.insert(
                key("posted-discovery", generation, 1),
                record(occurrence(), DemandState::Running, 1),
            );
        }
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::add_custody(
                &mut state,
                &receipts[..entries],
                neighbors,
            );
        let _published = SupersededSettlements::new_for_test(registry.clone(), Arc::new(selected.clone()));
        let before = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::occurrence_rows_visited();
        assert!(after_publication(&state, &selected).is_empty());
        contacts.push(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::occurrence_rows_visited() - before);
    }
    assert_eq!(contacts, [3, 3]);
}
