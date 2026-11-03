//! Supersession releases each retired row's own prepaid claims.
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::ready_backing::PreparedReadyBacking;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryPendingOutputDelivery;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

#[test]
fn an_incomparable_abandoned_publication_refunds_its_prerequisites() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let occurrence = occurrence();
    worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|_| {
        let mut state = DemandRegistryState::default();
        let unrelated = crate::domain_computation::primary_graph::application_output_demand::registry::tests::reservation_baseline::install(&mut state, receipt.clone());
        let upstream = key("abandoned-upstream", 1, 2);
        state.records.insert(
            upstream.clone(),
            record(occurrence, DemandState::Admitted, 1),
        );

        let abandoned = key_with_identity("abandoned-binding", 2, 1, 200);
        let incoming = key_with_identity("incoming-binding", 2, 1, 201);
        assert_eq!(
            abandoned.replacement_order(&incoming),
            None,
            "different Preserve bindings do not establish a succession"
        );
        let backing = PreparedReadyBacking::new();
        let output = WorthQueryOutputProgress::new(WorthQueryOutputCheckpoint::Published {
            receipt,
            delivery: WorthQueryPendingOutputDelivery::NoChange,
            ready_backing: backing,
        });
        let mut abandoned_record = record(occurrence, DemandState::Output(output), 0);
        abandoned_record.prerequisites = vec![Arc::new(upstream.clone())];

        state
            .records
            .get_mut(&upstream)
            .unwrap()
            .framework_required_count = 1;
        state.records.insert(abandoned.clone(), abandoned_record);
        let member = state.prepare_required_member(&abandoned).unwrap();
        state.install_required_member(member);
        supersede_predecessors(&mut state, &incoming).unwrap();
        assert!(state.records.contains_key(&unrelated));
        assert!(!state.records.contains_key(&abandoned));
        assert_eq!(
            state.records[&upstream].framework_required_count, 0,
            "retiring the incomparable row releases its exact consumed edge"
        );
    });
}
