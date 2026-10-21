//! Supersession releases each retired row's own prepaid claims.
use super::super::PreparedReadyBacking;
use super::super::WorthQueryOutputCheckpoint;
use super::super::WorthQueryOutputProgress;
use super::super::WorthQueryPendingOutputDelivery;
use super::*;

#[test]
fn an_incomparable_abandoned_publication_refunds_its_prerequisites() {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let occurrence = occurrence();
    crate::domain_computation::primary_graph::with_test_advancement(|_| {
        let mut state = DemandRegistryState::default();
        let unrelated = super::reservation_baseline::install(&mut state, receipt.clone());
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
