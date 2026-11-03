use crate::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds;
    use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::DemandRegistryState;
    use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::DemandState;
    use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
    use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::post_commit::custody_reader::resolve_before_binding;
    use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;

#[test]
fn binding_reader_drains_only_its_named_queued_custody() {
    let receipts = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::receipt_batch();
    let mut state = DemandRegistryState::default();
    worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::add_custody(&mut state, &receipts[..4], 1);
    for mut row in state.records.values_mut() {
        row.interests = 0;
        row.state = DemandState::Failed(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::NoEffect,
            "completed reader row",
        ));
    }
    for generation in 1..=64 {
        state.records.insert(
                worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key("unrelated", generation, 2),
                worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::record(
                    receipts[0].product_branch().occurrence(),
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
    assert_eq!(state.source_custody.len(), 4);
    assert_eq!(state.custody_cleanup.len, 4);
    for i in 0..4 {
        assert!(state.source_custody[commit(i)].cleanup.queued);
    }
    resolve_before_binding(&mut state, &receipts[2]);
    assert!(!state.source_custody.contains_key(commit(2)));
    for i in [0, 1, 3] {
        assert!(
            state.source_custody.contains_key(commit(i)),
            "unrelated custody stays owned"
        );
        assert!(state.source_custody[commit(i)].cleanup.queued);
    }
    assert_eq!(
        state.custody_cleanup.len, 3,
        "only the named middle entry was drained"
    );
    assert_eq!(
        state
            .records
            .values()
            .filter(|row| row.interests == 1)
            .count(),
        64,
        "binding cleanup preserves unrelated live rows"
    );
}
