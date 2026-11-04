use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
    use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::record;

use std::sync::Arc;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::PreparedOutputRootKind;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::SourceCustody;
    use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
    use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest;
    use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandNotifications;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;
    use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support;
#[test]
fn registry_product_retirement_prunes_rows_and_their_notes_together() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let registry = WorthQueryOutputDemandRegistry::default();
    let held = key("retired-note", 1, 2);
    let wake = {
        let mut state = (*registry.state_for_test()).lock().unwrap();
        for slot in 1..=3 {
            let key = key("retired-note", 1, slot);
            state.records.insert(
                key.clone(),
                record(
                    receipt.product_branch().occurrence_for_test(),
                    DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                        WorthQueryOutputDemandDenialKind::NoEffect,
                        "terminal debt",
                    )),
                    usize::from(slot == 2),
                ),
            );
            let member = state.prepare_required_member(&key).unwrap();
            state.install_required_member(member);
            state.note_closed_row(&key);
        }
        Arc::clone(&state.records[&held].wake)
    };
    let before = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::post_commit::retirement_observation::sweep_contacts();
    registry.release_product_occurrence_for_test(receipt.product_branch().occurrence_for_test());
    // Two lifecycle and two prerequisite-custody passes over three rows.
    // Cleanup notes remain separate.
    assert_eq!(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::post_commit::retirement_observation::sweep_contacts() - before,
            12
        );
    {
        let state = (*registry.state_for_test()).lock().unwrap();
        assert_eq!(state.records.len(), 1);
        assert_eq!(state.closed_cleanup.len(), 3);
        assert!(state.closed_cleanup.valid_order());
    }
    drop(WorthQueryOutputDemandInterest {
        key: held,
        requires_output: false,
        notifications: WorthQueryOutputDemandNotifications { wake },
        owner: registry.clone(),
    });
    registry.drain_closed_cleanup_admitted().unwrap();
    let state = (*registry.state_for_test()).lock().unwrap();
    assert!(state.records.is_empty());
    assert_eq!(state.closed_cleanup.len(), 0);
    assert!(state.closed_cleanup.head().is_none());
}

#[test]
fn registry_product_retirement_sweep_names_its_registry_bound() {
    let receipts: Vec<_> = (0..5)
            .map(|_| worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application())
            .collect();
    let mut contacts = Vec::new();
    for (rows, custody_entries) in [(2, 1), (5, 1), (9, 4)] {
        let registry = WorthQueryOutputDemandRegistry::default();
        let mut state = (*registry.state_for_test()).lock().unwrap();
        for generation in 1..=rows {
            state.records.insert(
                key("retired", generation, 1),
                record(
                    receipts[0].product_branch().occurrence_for_test(),
                    DemandState::Admitted,
                    1,
                ),
            );
        }
        for receipt in receipts.iter().take(custody_entries) {
            state.source_custody.insert(
                receipt
                    .committed_product_publication()
                    .composite_commit()
                    .clone(),
                SourceCustody {
                    occurrence: receipt.product_branch().occurrence_for_test(),
                    root_kind: PreparedOutputRootKind::Discovered(
                        worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId::of_for_test::<()>(),
                    ),
                    source: None,
                    discovery: None,
                    bound_sources: None,
                    consumed_sources: Vec::new(),
                    retired_sources: Vec::new(),
                    retired: None,
                    token_count: 1,
                    cleanup: Default::default(),
                    completed: false,
                },
            );
        }
        drop(state);
        let before = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::post_commit::retirement_observation::sweep_contacts();
        registry.release_product_occurrence_for_test(receipts[0].product_branch().occurrence_for_test());
        contacts.push(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::post_commit::retirement_observation::sweep_contacts() - before);
        assert!((*registry
            .state_for_test())
            .lock()
            .unwrap()
            .records
            .values()
            .all(|row| row.terminal()));
    }
    // Two lifecycle and two prerequisite-custody passes, plus each custody entry.
    assert_eq!(contacts, [4 * 2 + 1, 4 * 5 + 1, 4 * 9 + 4]);
}
