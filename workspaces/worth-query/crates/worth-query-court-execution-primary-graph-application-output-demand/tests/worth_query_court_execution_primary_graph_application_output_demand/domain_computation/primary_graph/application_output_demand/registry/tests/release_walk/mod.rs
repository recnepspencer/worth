//! One released edge inspects each released row's awaiting owners once.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::PreparedReadyBacking;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryPendingOutputDelivery;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

#[test]
fn a_released_claim_walks_each_released_key_once() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let occurrence = occurrence();
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|_| {
        let mut state = DemandRegistryState::default();
        let stale = key("released-occurrence", 1, 1);
        let newest = key("released-occurrence", 2, 1);
        let dependent = key("closing-dependent", 1, 2);
        let mut stale_row = record(
            occurrence,
            DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                WorthQueryOutputDemandDenialKind::Superseded,
                "",
            )),
            0,
        );
        (*stale_row.framework_required_count_mut_for_test()) = 1;
        (*state.records_for_test()).insert_for_test(stale.clone(), stale_row);
        let backing = PreparedReadyBacking::new_for_test();
        (*state.records_for_test()).insert_for_test(
            newest.clone(),
            record(
                occurrence,
                DemandState::Output(WorthQueryOutputProgress::new_for_test(
                    WorthQueryOutputCheckpoint::Published {
                        receipt,
                        delivery: WorthQueryPendingOutputDelivery::NoChange,
                        ready_backing: backing,
                    },
                )),
                0,
            ),
        );
        for key in [&stale, &newest] {
            let member = state.prepare_required_member_for_test(key).unwrap();
            state.install_required_member_for_test(member);
        }
        let mut row = record(
            occurrence,
            DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                WorthQueryOutputDemandDenialKind::Closed,
                "",
            )),
            0,
        );
        (*row.prerequisites_mut_for_test()) = vec![Arc::new(stale.clone())];

        (*state.records_for_test()).insert_for_test(dependent.clone(), row);
        let before = (*state.records_for_test()).occurrence_visits_for_test();
        drop(state.release_record_prerequisites_for_test(&dependent));
        let visited = (*state.records_for_test()).occurrence_visits_for_test() - before;
        // Two rows: one successor search and one awaited-owner walk for each
        // of the stale and newest keys. No defer or membership rewalk is owed.
        assert_eq!(visited, 2 + 2 + 2, "released-key occurrence contacts");
        assert_eq!((*(*state.records_for_test())[&stale].framework_required_count_for_test()), 0);
        assert!((*(*state.records_for_test())[&newest].pending_cleanup_queued_for_test()));
    });
}
