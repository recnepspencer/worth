//! Supersession releases each retired row's own prepaid claims.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::PreparedReadyBacking;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryPendingOutputDelivery;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

#[test]
fn an_incomparable_abandoned_publication_refunds_its_prerequisites() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let occurrence = occurrence();
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|_| {
        let mut state = DemandRegistryState::default();
        let unrelated = crate::domain_computation::primary_graph::application_output_demand::registry::tests::reservation_baseline::install(&mut state, receipt.clone());
        let upstream = key("abandoned-upstream", 1, 2);
        (*state.records_for_test()).insert_for_test(
            upstream.clone(),
            record(occurrence, DemandState::Admitted, 1),
        );

        let abandoned = key_with_identity("abandoned-binding", 2, 1, 200);
        let incoming = key_with_identity("incoming-binding", 2, 1, 201);
        assert_eq!(
            abandoned.replacement_order_for_test(&incoming),
            None,
            "different Preserve bindings do not establish a succession"
        );
        let backing = PreparedReadyBacking::new_for_test();
        let output = WorthQueryOutputProgress::new_for_test(WorthQueryOutputCheckpoint::Published {
            receipt,
            delivery: WorthQueryPendingOutputDelivery::NoChange,
            ready_backing: backing,
        });
        let mut abandoned_record = record(occurrence, DemandState::Output(output), 0);
        (*abandoned_record.prerequisites_mut_for_test()) = vec![Arc::new(upstream.clone())];

        (*(*state
            .records_for_test())
            .get_mut(&upstream)
            .unwrap()
            .framework_required_count_mut_for_test()) = 1;
        (*state.records_for_test()).insert_for_test(abandoned.clone(), abandoned_record);
        let member = state.prepare_required_member_for_test(&abandoned).unwrap();
        state.install_required_member_for_test(member);
        supersede_predecessors(&mut state, &incoming).unwrap();
        assert!((*state.records_for_test()).contains_key_for_test(&unrelated));
        assert!(!(*state.records_for_test()).contains_key_for_test(&abandoned));
        assert_eq!(
            (*(*state.records_for_test())[&upstream].framework_required_count_for_test()), 0,
            "retiring the incomparable row releases its exact consumed edge"
        );
    });
}
