use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandRegistryState;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;
    use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::post_commit::custody_reader::resolve_before_binding;
    use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;

#[test]
fn binding_reader_drains_only_its_named_queued_custody() {
    let receipts = (0..3).map(|_| worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application()).collect::<Vec<_>>();
    let mut state = DemandRegistryState::default();
    worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::add_custody(&mut state, &receipts, 1);
    for mut row in (*state.records_mut_for_test()).values_mut_for_test() {
        (*row.interests_mut_for_test()) = 0;
        (*row.state_mut_for_test()) =
            DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                WorthQueryOutputDemandDenialKind::NoEffect,
                "completed reader row",
            ));
    }
    for generation in 1..=2 {
        (*state.records_mut_for_test()).insert_for_test(
                worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key("unrelated", generation, 2),
                worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::record(
                    receipts[0].product_branch().occurrence_for_test(),
                    DemandState::Admitted,
                    1,
                ),
            );
    }
    let commit = |i: usize| {
        receipts[i]
            .committed_product_publication()
            .composite_commit()
    };
    assert_eq!(state.source_custody.len(), 3);
    assert_eq!(state.custody_cleanup_for_test().len, 3);
    for i in 0..3 {
        assert!(state.source_custody[commit(i)].cleanup_for_test().queued);
    }
    resolve_before_binding(&mut state, commit(1));
    assert!(!state.source_custody.contains_key(commit(1)));
    for i in [0, 2] {
        assert!(
            state.source_custody.contains_key(commit(i)),
            "unrelated custody stays owned"
        );
        assert!(state.source_custody[commit(i)].cleanup_for_test().queued);
    }
    assert_eq!(
        state.custody_cleanup_for_test().len,
        2,
        "only the named middle entry was drained"
    );
    assert_eq!(
        (*state.records_for_test())
            .values_for_test()
            .filter(|row| (*row.interests_for_test()) == 1)
            .count(),
        2,
        "binding cleanup preserves unrelated live rows"
    );
}
