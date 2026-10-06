//! Exact local posting grouping and reverse-index witnesses.

use std::sync::Arc;

use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    mvcc::CompanionPreflightBudget,
};

use super::{
    super::super::{fact_key::FactPostingKey, mark_state::MarkState},
    group_projected, InvalidationEditAdmission, PreparedPostingOrdinals, ProjectedKey,
    WorthQueryApplicationObservedFact as Fact,
};

#[test]
fn equal_fingerprints_keep_distinct_exact_keys_and_merge_only_equal_keys() {
    let first = Arc::new(FactPostingKey::EntityLifecycle(EntityId::new(
        PartitionId::main(),
        1,
        1,
    )));
    let second = Arc::new(FactPostingKey::EntityLifecycle(EntityId::new(
        PartitionId::main(),
        2,
        1,
    )));
    let projected = vec![
        ProjectedKey {
            hash: 7,
            key: Arc::clone(&first),
            ordinal: 0,
        },
        ProjectedKey {
            hash: 7,
            key: Arc::clone(&second),
            ordinal: 1,
        },
        ProjectedKey {
            hash: 7,
            key: Arc::clone(&first),
            ordinal: 2,
        },
    ];
    let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: 10_000,
        maximum_preparation_bytes: 64 * 1024 * 1024,
    });
    let groups = group_projected(projected, 3, &mut admission).unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].key.as_ref(), first.as_ref());
    assert_eq!(
        groups[0].ordinals.iter().copied().collect::<Vec<_>>(),
        vec![0, 2]
    );
    assert_eq!(groups[1].key.as_ref(), second.as_ref());
    assert_eq!(
        groups[1].ordinals.iter().copied().collect::<Vec<_>>(),
        vec![1]
    );
}

#[test]
fn source_and_output_facts_share_one_key_but_keep_both_composite_ordinals() {
    let (_, identity) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let source = [Fact::SourceEntity { entity_id: entity }];
    let output = [Fact::Entity {
        entity_id: entity,
        kind: KindId::new(9),
    }];
    let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: 10_000,
        maximum_preparation_bytes: 64 * 1024 * 1024,
    });
    let prepared =
        PreparedPostingOrdinals::prepare(&source, Some(&output), &mut None, &mut admission)
            .unwrap();
    let mut state = MarkState::initial();
    let (ordinals, _) = prepared
        .install(&mut state, &identity, &mut admission)
        .unwrap();
    let key = FactPostingKey::EntityLifecycle(entity);
    let own = ordinals.get(&key).unwrap();
    assert_eq!(own.iter().copied().collect::<Vec<_>>(), vec![0, 1]);
    let global = state.postings.get(&key).unwrap();
    assert_eq!(global.len(), 2);
    assert_eq!(
        global
            .iter()
            .map(|posting| posting.ordinal)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert!(global
        .iter()
        .all(|posting| Arc::ptr_eq(&posting.settlement, &identity)));
}
