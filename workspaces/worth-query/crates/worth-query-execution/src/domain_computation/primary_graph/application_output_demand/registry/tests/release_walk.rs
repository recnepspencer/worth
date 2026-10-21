//! One released edge inspects each released row's awaiting owners once.
use super::super::PreparedReadyBacking;
use super::super::WorthQueryOutputCheckpoint;
use super::super::WorthQueryOutputProgress;
use super::super::WorthQueryPendingOutputDelivery;
use super::*;

#[test]
fn a_released_claim_walks_each_released_key_once() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let occurrence = occurrence();
    crate::domain_computation::primary_graph::with_test_advancement(|_| {
        let mut state = DemandRegistryState::default();
        let stale = key("released-occurrence", 1, 1);
        let newest = key("released-occurrence", 2, 1);
        let dependent = key("closing-dependent", 1, 2);
        let mut stale_row = record(
            occurrence,
            DemandState::Failed(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Superseded,
                "",
            )),
            0,
        );
        stale_row.framework_required_count = 1;
        state.records.insert(stale.clone(), stale_row);
        let backing = PreparedReadyBacking::new();
        state.records.insert(
            newest.clone(),
            record(
                occurrence,
                DemandState::Output(WorthQueryOutputProgress::new(
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
            let member = state.prepare_required_member(key).unwrap();
            state.install_required_member(member);
        }
        let mut row = record(
            occurrence,
            DemandState::Failed(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Closed,
                "",
            )),
            0,
        );
        row.prerequisites = vec![Arc::new(stale.clone())];

        state.records.insert(dependent.clone(), row);
        let before = state.records.occurrence_visits_for_test();
        drop(state.release_record_prerequisites(&dependent));
        let visited = state.records.occurrence_visits_for_test() - before;
        // Two rows: one successor search and one awaited-owner walk for each
        // of the stale and newest keys. No defer or membership rewalk is owed.
        assert_eq!(visited, 2 + 2 + 2, "released-key occurrence contacts");
        assert_eq!(state.records[&stale].framework_required_count, 0);
        assert!(state.records[&newest].pending_cleanup_queued);
    });
}
