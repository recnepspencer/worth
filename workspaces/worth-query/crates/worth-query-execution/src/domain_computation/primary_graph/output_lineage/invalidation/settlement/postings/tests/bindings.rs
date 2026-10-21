//! Shared physical storage never substitutes one exact settlement for another.
use super::super::super::super::delivery::affected::posting_order::LogicalPostings;
use super::super::super::super::mark_state::FactPosting;
use super::super::super::posting_bindings;
use super::super::super::posting_removal;
use super::*;

fn projection(id: u64) -> Arc<super::super::super::PostingProjection> {
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let facts = [
        Fact::SourceEntity { entity_id: entity },
        Fact::SourceEntity { entity_id: entity },
    ];
    PreparedPostingOrdinals::prepare(&facts[..], None, &mut None)
        .unwrap()
        .into_projection(id, facts.len(), 0, None)
}

#[test]
fn shared_bindings_detach_exactly_and_historical_storage_lives_until_final_drop() {
    let (_, [first, second]) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement_pair();
    let projection = projection(0);
    let mut state = MarkState::initial();
    posting_bindings::bind(&mut state, &projection, &first).unwrap();
    let historical = state.clone();
    let physical = state.postings.clone();
    posting_bindings::bind(&mut state, &projection, &second).unwrap();
    assert_eq!(state.postings, physical);
    assert_eq!(
        state.posting_bindings[&0]
            .settlements()
            .unwrap()
            .settlements
            .len(),
        2
    );
    posting_removal::remove(&mut state, &first, &projection).unwrap();
    assert_eq!(state.postings, physical);
    assert_eq!(
        state.posting_bindings[&0]
            .settlements()
            .unwrap()
            .settlements
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
        vec![second.clone()]
    );
    assert!(historical.posting_bindings[&0]
        .settlements()
        .unwrap()
        .settlements
        .contains(&first));
    assert!(!historical.posting_bindings[&0]
        .settlements()
        .unwrap()
        .settlements
        .contains(&second));
    posting_removal::remove(&mut state, &second, &projection).unwrap();
    assert!(state.postings.is_empty());
    assert!(state.posting_bindings.is_empty());
    assert_eq!(state.posting_count, 0);
    drop(state);
    drop(projection);

    drop(historical);
}

#[test]
fn logical_delivery_orders_exact_settlements_and_duplicate_ordinals_not_projection_ids() {
    let (_, [first, second]) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement_pair();
    let mut state = MarkState::initial();
    let later = projection(0);
    let earlier = projection(1);
    posting_bindings::bind(&mut state, &later, &second).unwrap();
    posting_bindings::bind(&mut state, &earlier, &first).unwrap();
    let key = FactPostingKey::EntityLifecycle(EntityId::new(PartitionId::main(), 1, 1));
    let mut counts = Default::default();
    let mut cursor = LogicalPostings::prepare(&state, &key, &mut counts)
        .unwrap()
        .unwrap();
    let mut observed = Vec::new();
    while let Some(posting) = cursor.next(&state, &key, &mut counts).unwrap() {
        observed.push(posting);
    }
    assert_eq!(
        observed,
        vec![
            FactPosting {
                settlement: first.clone(),
                ordinal: 0
            },
            FactPosting {
                settlement: first,
                ordinal: 1
            },
            FactPosting {
                settlement: second.clone(),
                ordinal: 0
            },
            FactPosting {
                settlement: second,
                ordinal: 1
            }
        ]
    );
    assert_eq!(
        counts.matched_fact_postings as usize,
        observed.len(),
        "each actually discovered logical pair is counted once"
    );
}
