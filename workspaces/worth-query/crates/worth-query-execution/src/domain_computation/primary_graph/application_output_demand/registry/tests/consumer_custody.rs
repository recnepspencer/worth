//! A wave's funded interest protects a real row; reclamation grants no proof.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::registry_fixture;

#[test]
fn a_consumer_handoff_ends_at_real_row_reclamation() {
    let (_lineage, identity) = registry_fixture::recorded_settlement();
    let registry = WorthQueryOutputDemandRegistry::default();
    let consumer = Arc::new(key("handoff-consumer", 1, 1));
    {
        let mut state = registry.state.lock().unwrap();
        let (posting, _cancellation) = state
            .reserve_settlement_vacancy(&identity, &mut record_admission())
            .unwrap();
        super::super::settlement_index::SettlementIndex::fill_prepared(
            &posting,
            &identity,
            Arc::clone(&identity),
            Arc::clone(&consumer),
        );
        let mut row = record(identity.address().0, super::closed_retirement::ready(), 0);
        row.settlements.push((Arc::clone(&identity), 0));
        state.required_reserved_bytes += row.settlements.capacity()
            * std::mem::size_of::<(Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>, usize)>();
        state.records.insert(consumer.as_ref().clone(), row);
    }
    let handoff = registry
        .retain_consumer_handoff(&identity, &mut record_admission())
        .unwrap();
    assert!(handoff.matches(&identity));
    {
        let mut state = registry.state.lock().unwrap();
        assert_eq!(state.records[consumer.as_ref()].interests, 1);
        assert_eq!(state.records[consumer.as_ref()].framework_required_count, 0);
        state.required_budget_bytes = 0;
    }
    // The wave interest protects the posted row under real custody pressure.
    registry
        .reclaim_cached_rows(1, &mut record_admission())
        .unwrap();
    assert!(registry
        .state
        .lock()
        .unwrap()
        .records
        .contains_key(consumer.as_ref()));
    drop(handoff);
    registry
        .reclaim_cached_rows(1, &mut record_admission())
        .unwrap();
    assert!(!registry
        .state
        .lock()
        .unwrap()
        .records
        .contains_key(consumer.as_ref()));
    assert!(!registry
        .posts_settlement(&identity, &mut record_admission())
        .unwrap());
    let released = registry
        .retain_consumer_handoff(&identity, &mut record_admission())
        .unwrap();
    assert!(
        !released.matches(&identity),
        "a reclaimed row owns no equality proof"
    );
}
