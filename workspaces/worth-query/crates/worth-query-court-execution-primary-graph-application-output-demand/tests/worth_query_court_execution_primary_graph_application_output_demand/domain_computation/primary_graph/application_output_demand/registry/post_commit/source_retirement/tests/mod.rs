use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
    use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::record;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandRegistryState;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::post_commit::source_retirement::retire_stale_records;
#[test]
fn source_retirement_closes_all_producers_and_preserves_custody() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let scope = receipt.principal_scope().scope();
    let occurrence = receipt.product_branch().occurrence_for_test();
    let receipts = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::receipt_batch();
    for producers in [1, 3, 7] {
        let mut state = DemandRegistryState::default();
        for producer in 0..producers {
            let mut row = record(occurrence, DemandState::Admitted, 1);
            (*row.source_scope_mut_for_test()) = Some(scope);
            (*state.records_mut_for_test())
                .insert_for_test(key(&format!("source-owner-{producer}"), 1, 1), row);
        }
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::add_custody(&mut state, &receipts, 3);
        retire_stale_records(
            &mut state,
            occurrence,
            scope,
            key("unused", 2, 1).source_for_test(),
        );
        let selected = key("unused", 2, 1);
        let selected_rows = (*state.records_for_test())
            .source_occurrence_rows_for_test(selected.source_for_test())
            .collect::<Vec<_>>();
        assert_eq!(selected_rows.len(), producers);
        assert!(selected_rows.iter().all(|(_, row)| row.terminal_for_test()));
        assert_eq!(
            (*state.records_for_test())
                .values_for_test()
                .filter(|row| !row.terminal_for_test())
                .count(),
            receipts.len() * 3
        );
        assert_eq!(state.source_custody.len(), receipts.len());
    }
}
