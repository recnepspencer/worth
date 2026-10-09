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

#[test]
fn final_posting_storage_keeps_all_ordinals_without_retaining_grouping_scratch() {
    use super::super::super::index_capacity;
    for (keys, copies) in [(1usize, 1usize), (1, 128), (128, 1)] {
        let facts: Vec<_> = (0..keys)
            .flat_map(|key| {
                std::iter::repeat_n(
                    Fact::SourceEntity {
                        entity_id: EntityId::new(PartitionId::main(), key as u64 + 1, 1),
                    },
                    copies,
                )
            })
            .collect();
        let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: 1_000_000,
            maximum_preparation_bytes: 64 * 1024 * 1024,
        });
        let prepared =
            PreparedPostingOrdinals::prepare(&facts, None, &mut None, &mut admission).unwrap();
        assert_eq!(prepared.ordinals.len(), keys);
        assert!(prepared
            .ordinals
            .values()
            .all(|ordinals| ordinals.len() == copies));
        let expected =
            index_capacity::retained_map_bytes::<Arc<FactPostingKey>, im::OrdSet<usize>>(keys)
                .unwrap()
                + index_capacity::retained_forest_bytes::<usize, ()>(keys * copies, keys).unwrap()
                + (keys as u64) * index_capacity::arc_bytes::<FactPostingKey>().unwrap();
        assert_eq!(admission.charged_index_bytes(), expected);
        assert!(admission.charged_bytes() > expected);
        let (_, identity) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
        let mut state = MarkState::initial();
        let prior = state.postings.clone();
        let before_index = admission.charged_index_bytes();
        let before_preparation = admission.charged_bytes();
        prepared
            .install(&mut state, &identity, &mut admission)
            .unwrap();
        assert!(prior.is_empty());
        assert_eq!(state.posting_count, keys * copies);
        assert_eq!(state.postings.len(), keys);
        assert!(state.postings.values().all(|postings| {
            postings.len() == copies
                && postings
                    .iter()
                    .all(|posting| Arc::ptr_eq(&posting.settlement, &identity))
        }));
        // The new global tree and every independently rooted posting bucket
        // fit their final stable trees, even after many paths were prepared.
        let whole = index_capacity::retained_map_bytes::<
            Arc<FactPostingKey>,
            im::OrdSet<super::super::super::mark_state::FactPosting>,
        >(keys)
        .unwrap()
            + keys as u64
                * index_capacity::retained_map_bytes::<
                    super::super::super::mark_state::FactPosting,
                    (),
                >(copies)
                .unwrap();
        let installed = admission.charged_index_bytes() - before_index;
        assert!(installed > 0 && installed <= whole);
        assert!(admission.charged_bytes() - before_preparation > installed);
    }
}
