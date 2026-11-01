//! Pure storage proofs; original node owners are observed through real walks.
use super::*;
use crate::identity::data::PartitionId;
use crate::storage::partition::SparseAdjacencyTable;
use crate::storage::substrate::{
    StorageAllocationObservation, StorageAllocationVisitor, StoredValue,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct Owners(BTreeMap<u64, u64>);
impl StorageAllocationVisitor for Owners {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        if let Some(old) = self.0.insert(allocation.id, allocation.bytes) {
            assert_eq!(old, allocation.bytes);
            false
        } else {
            true
        }
    }
}
fn owners(table: &SparseAdjacencyTable) -> Owners {
    let mut owners = Owners::default();
    table.visit_authoritative_allocations(true, &mut owners);
    table.visit_cache_allocations(&mut owners);
    owners
}
fn policy(backend: AdjacencyBackend) -> AdjacencyPolicy {
    AdjacencyPolicy {
        backend,
        small_degree_inline_capacity: 4,
    }
}
fn relation(slot: u64) -> RelationId {
    RelationId::new(PartitionId::main(), slot, 1)
}
fn node_count(set: &AdjacencySet) -> usize {
    let entries = set.entries();
    entries.current.len()
        + [&entries.current_by_kind, &entries.historical_by_kind]
            .into_iter()
            .flatten()
            .map(|kinds| kinds.len() + kinds.values().map(RelationSet::len).sum::<usize>())
            .sum::<usize>()
        + entries
            .structural_revision_by_kind
            .as_ref()
            .map_or(0, SharedMap::len)
}
#[test]
fn inline_adjacency_headers_preserve_retained_current_and_historical_roots() {
    for backend in [
        AdjacencyBackend::InlineSmallDegreeAdjacency,
        AdjacencyBackend::CompressedFanoutAdjacency,
    ] {
        let policy = policy(backend);
        let mut table = SparseAdjacencyTable::default();
        let set = table.ensure(11, &policy);
        set.insert_at(KindId(7), relation(1), VersionId(1));
        set.insert_at(KindId(8), relation(2), VersionId(1));
        let original = table.clone();
        let set = table.get_mut(11).unwrap();
        set.remove_at(KindId(7), &relation(1), VersionId(2));
        set.insert_at(KindId(7), relation(3), VersionId(3));
        let old = original.get(11).unwrap();
        let current = table.get(11).unwrap();
        assert_eq!(old.current_kind_ids(KindId(7)), [relation(1)]);
        assert_eq!(current.current_kind_ids(KindId(7)), [relation(3)]);
        assert_eq!(old.historical_kind_ids(KindId(7)), [relation(1)]);
        assert_eq!(
            current.historical_kind_ids(KindId(7)),
            [relation(1), relation(3)]
        );
        assert_eq!(old.structural_revision(KindId(7)), Some(VersionId(1)));
        assert_eq!(current.structural_revision(KindId(7)), Some(VersionId(3)));
        let old_unrelated = old
            .entries()
            .current_by_kind
            .as_ref()
            .unwrap()
            .get(&KindId(8))
            .unwrap();
        let new_unrelated = current
            .entries()
            .current_by_kind
            .as_ref()
            .unwrap()
            .get(&KindId(8))
            .unwrap();
        assert!(old_unrelated.same_storage(new_unrelated));
        assert!(!old.same_storage(current));
        let retained = current.clone();
        drop(table);
        assert_eq!(retained.current_kind_ids(KindId(7)), [relation(3)]);
    }
}
#[test]
fn inline_header_allocation_walk_matches_actual_child_nodes_and_delta() {
    let policy = policy(AdjacencyBackend::InlineSmallDegreeAdjacency);
    let mut old = SparseAdjacencyTable::default();
    old.ensure(11, &policy)
        .insert_at(KindId(7), relation(1), VersionId(1));
    old.ensure(19, &policy)
        .insert_at(KindId(8), relation(2), VersionId(1));
    let before = owners(&old);
    let expected_nodes =
        old.iter().count() + old.iter().map(|(_, set)| node_count(set)).sum::<usize>();
    assert_eq!(before.0.len(), expected_nodes);
    let expected_bytes = old.allocation_bytes()
        + old
            .iter()
            .map(|(_, set)| {
                set.authoritative_allocation_bytes() + set.optional_cache_allocation_bytes()
            })
            .sum::<u64>();
    assert_eq!(before.0.values().sum::<u64>(), expected_bytes);
    assert_eq!(<AdjacencySet as StoredValue>::allocation_bytes(), 0);
    assert_eq!(<RelationSet as StoredValue>::allocation_bytes(), 0);
    let mut current = old.clone();
    current
        .get_mut(11)
        .unwrap()
        .remove_at(KindId(7), &relation(1), VersionId(2));
    current
        .get_mut(11)
        .unwrap()
        .insert_at(KindId(7), relation(3), VersionId(3));
    let after = owners(&current);
    let expected = after
        .0
        .keys()
        .filter(|id| !before.0.contains_key(id))
        .copied()
        .collect::<BTreeSet<_>>();
    let mut actual = Owners::default();
    current.visit_new_authoritative_allocations(&old, &mut actual);
    current.visit_new_cache_allocations(&old, &mut actual);
    assert_eq!(actual.0.keys().copied().collect::<BTreeSet<_>>(), expected);
    for (id, bytes) in &actual.0 {
        assert_eq!(after.0.get(id), Some(bytes));
    }
    assert_eq!(owners(&old).0, before.0);
}
#[test]
fn inline_adjacency_views_preserve_legacy_named_and_compact_sequence_wire() {
    let mut set = AdjacencySet::new(&policy(AdjacencyBackend::CompressedFanoutAdjacency));
    set.insert_at(KindId(7), relation(3), VersionId(1));
    set.insert_at(KindId(7), relation(1), VersionId(1));
    set.remove_at(KindId(7), &relation(3), VersionId(2));
    let current = vec![relation(1)];
    let historical = vec![relation(1), relation(3)];
    assert_eq!(
        rmp_serde::to_vec(&set.current_ids()).unwrap(),
        rmp_serde::to_vec(&current).unwrap()
    );
    assert_eq!(
        rmp_serde::to_vec_named(&set.current_ids()).unwrap(),
        rmp_serde::to_vec_named(&current).unwrap()
    );
    assert_eq!(
        rmp_serde::to_vec(&set.historical_kind_ids(KindId(7))).unwrap(),
        rmp_serde::to_vec(&historical).unwrap()
    );
    assert_eq!(
        rmp_serde::to_vec_named(&set.historical_kind_ids(KindId(7))).unwrap(),
        rmp_serde::to_vec_named(&historical).unwrap()
    );
}
