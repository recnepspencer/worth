//! Shared physical storage never substitutes one exact settlement for another.
use super::super::super::super::{
    delivery::affected::posting_order::LogicalPostings, mark_state::FactPosting,
};
use super::super::super::{posting_bindings, posting_removal};
use super::*;

fn admission() -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: 1_000_000,
        maximum_preparation_bytes: 64 * 1024 * 1024,
    })
}
fn projection(
    id: u64,
    resources: &crate::domain_computation::execution_runtime::source_invalidation::WorthQueryInvalidationResources,
    admission: &mut InvalidationEditAdmission,
) -> Arc<super::super::super::PostingProjection> {
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let facts = [
        Fact::SourceEntity { entity_id: entity },
        Fact::SourceEntity { entity_id: entity },
    ];
    PreparedPostingOrdinals::prepare(&facts[..], None, &mut None, admission)
        .unwrap()
        .into_projection(id, facts.len(), 0, None, resources, admission)
        .unwrap()
}

#[test]
fn shared_bindings_detach_exactly_and_historical_storage_lives_until_final_drop() {
    let (_, [first, second]) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement_pair();
    let resources = resources();
    let mut admission = admission();
    let projection = projection(0, &resources, &mut admission);
    let mut state = MarkState::initial();
    posting_bindings::bind(&mut state, &projection, &first, &mut admission).unwrap();
    let historical = state.clone();
    let physical = state.postings.clone();
    posting_bindings::bind(&mut state, &projection, &second, &mut admission).unwrap();
    assert_eq!(state.postings, physical);
    assert_eq!(state.posting_bindings[&0].settlements.len(), 2);
    posting_removal::remove(&mut state, &first, &projection, &mut admission).unwrap();
    assert_eq!(state.postings, physical);
    assert_eq!(
        state.posting_bindings[&0]
            .settlements
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
        vec![second.clone()]
    );
    assert!(historical.posting_bindings[&0].settlements.contains(&first));
    assert!(!historical.posting_bindings[&0]
        .settlements
        .contains(&second));
    posting_removal::remove(&mut state, &second, &projection, &mut admission).unwrap();
    assert!(state.postings.is_empty());
    assert!(state.posting_bindings.is_empty());
    assert_eq!(state.posting_count, 0);
    drop(state);
    drop(projection);
    assert!(
        resources.retained_capacity_bytes() > 0,
        "the historical image holds the actual projection grant"
    );
    drop(historical);
    assert_eq!(resources.retained_capacity_bytes(), 0);
}

#[test]
fn logical_delivery_orders_exact_settlements_and_duplicate_ordinals_not_projection_ids() {
    let (_, [first, second]) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement_pair();
    let resources = resources();
    let mut admission = admission();
    let mut state = MarkState::initial();
    let later = projection(0, &resources, &mut admission);
    let earlier = projection(1, &resources, &mut admission);
    posting_bindings::bind(&mut state, &later, &second, &mut admission).unwrap();
    posting_bindings::bind(&mut state, &earlier, &first, &mut admission).unwrap();
    let key = FactPostingKey::EntityLifecycle(EntityId::new(PartitionId::main(), 1, 1));
    let mut counts = Default::default();
    let mut cursor = LogicalPostings::prepare(&state, &key, &mut admission, &mut counts)
        .unwrap()
        .unwrap();
    let mut observed = Vec::new();
    while let Some(posting) = cursor
        .next(&state, &key, &mut admission, &mut counts)
        .unwrap()
    {
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
    let mut counts = Default::default();
    let mut cursor = LogicalPostings::prepare(&state, &key, &mut admission, &mut counts)
        .unwrap()
        .unwrap();
    let mut stopped = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: 0,
        maximum_preparation_bytes: 0,
    });
    assert!(cursor
        .next(&state, &key, &mut stopped, &mut counts)
        .is_err());
    assert_eq!(state.posting_bindings.len(), 2);
    assert_eq!(
        cursor
            .next(&state, &key, &mut admission, &mut counts)
            .unwrap(),
        Some(observed[0].clone()),
        "a first stop does not consume a logical prefix"
    );
}

#[test]
fn a_refused_successor_binding_leaves_the_original_image_usable() {
    let (_, [first, second]) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement_pair();
    let resources = resources();
    let mut admission = admission();
    let projection = projection(0, &resources, &mut admission);
    let mut original = MarkState::initial();
    posting_bindings::bind(&mut original, &projection, &first, &mut admission).unwrap();
    let mut candidate = original.clone();
    let mut stopped = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: 0,
        maximum_preparation_bytes: 0,
    });
    assert!(posting_bindings::bind(&mut candidate, &projection, &second, &mut stopped).is_err());
    drop(candidate);
    assert_eq!(original.posting_bindings[&0].settlements.len(), 1);
    assert!(original.posting_bindings[&0].settlements.contains(&first));
    assert!(!original.postings.is_empty());
}
