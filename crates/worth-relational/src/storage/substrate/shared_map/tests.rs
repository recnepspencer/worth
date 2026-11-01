use super::*;
use proptest::prelude::*;
use std::collections::BTreeMap;

proptest! {
    #[test]
    fn persistent_edits_match_an_independent_ordered_map(actions in prop::collection::vec((0u16..200, any::<bool>(), any::<u64>()), 0..500)) {
        let mut map = SharedMap::<u16,u64>::new();
        let mut expected = BTreeMap::new();
        for (key, insert, value) in actions {
            let retained = map.clone();
            let before = expected.clone();
            if insert { map.insert(key, value); expected.insert(key, value); }
            else { prop_assert_eq!(map.remove(&key).0, expected.remove(&key).is_some()); }
            prop_assert!(map.iter().eq(expected.iter()));
            prop_assert!(retained.iter().eq(before.iter()));
            prop_assert_eq!(map.len(), expected.len());
            prop_assert_eq!(map.last_key_value(), expected.last_key_value());
            for probe in [0, key, key.saturating_add(1), 200] {
                prop_assert_eq!(map.lower_bound(&probe), expected.range(probe..).next());
                prop_assert_eq!(retained.lower_bound(&probe), before.range(probe..).next());
            }
            assert_balanced(&map.root);
            assert_balanced(&retained.root);
        }
    }
}

fn assert_balanced<K: Ord + Copy, V: StoredValue>(
    root: &Option<Arc<node::MapNode<K, V>>>,
) -> (usize, usize) {
    let Some(node) = root else { return (0, 0) };
    let (left_height, left_len) = assert_balanced(&node.left);
    let (right_height, right_len) = assert_balanced(&node.right);
    assert!(left_height.abs_diff(right_height) <= 1);
    assert_eq!(node.height, 1 + left_height.max(right_height));
    assert_eq!(node.len, 1 + left_len + right_len);
    (node.height, node.len)
}

#[test]
fn cold_sorted_construction_matches_ordered_map_and_remains_mutable() {
    for keys in [
        vec![],
        vec![0],
        (0..255).collect(),
        (0..255).map(|key| key * 17).collect(),
    ] {
        let values = keys
            .into_iter()
            .map(|key| (key, key + 1))
            .collect::<Vec<_>>();
        let expected = values.iter().copied().collect::<BTreeMap<_, _>>();
        let mut map = SharedMap::<u64, u64>::from_sorted_unique(values);
        assert!(map.iter().eq(expected.iter()));
        assert_balanced(&map.root);
        map.insert(10_000, 10_001);
        assert_eq!(map.get(&10_000), Some(&10_001));
        assert_balanced(&map.root);
    }
}

#[test]
fn removed_transient_nodes_still_count_as_copy_work() {
    let original: SharedMap<u64, u64> = [(1, 10), (2, 20)].into_iter().collect();
    let mut changed = original.clone();
    // Removing a shared root with one child clones then discards that root.
    // The resulting map reaches only old allocations; retained-owner deltas
    // alone would incorrectly report that no copying happened.
    assert_eq!(
        changed.remove(&1),
        (true, std::mem::size_of::<MapNode<u64, u64>>() as u64)
    );
    assert_eq!(
        changed.root.as_ref().unwrap().id(),
        original.root.as_ref().unwrap().right.as_ref().unwrap().id()
    );
    assert_eq!(original.len(), 2);
    drop(original);
    assert_eq!(changed.insert(2, 30), 0);
    assert_eq!(changed.insert(3, 40), 0);
}

#[test]
fn replacing_and_removing_retained_values_never_clone_displaced_payloads() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Counted(Arc<AtomicUsize>);
    impl Clone for Counted {
        fn clone(&self) -> Self {
            self.0.fetch_add(1, Ordering::Relaxed);
            Self(self.0.clone())
        }
    }
    let clones = Arc::new(AtomicUsize::new(0));
    let mut map: SharedMap<_, crate::storage::substrate::StorageAllocation<Counted>> = (0..1024)
        .map(|key| (key, Counted(clones.clone())))
        .collect();
    let retained = map.clone();
    for key in 0..512 {
        map.insert(key, Counted(clones.clone()));
    }
    for key in 512..1024 {
        assert!(map.remove(&key).0);
    }
    assert_eq!(clones.load(Ordering::Relaxed), 0);
    assert_eq!(retained.len(), 1024);
    assert_eq!(map.len(), 512);
}

#[test]
fn targeted_mutation_does_not_change_a_retained_map() {
    let mut map: SharedMap<_, crate::storage::substrate::StorageAllocation<String>> =
        (0..10_000).map(|key| (key, key.to_string())).collect();
    let retained = map.clone();
    map.get_mut(&5_000).unwrap().push('!');
    assert_eq!(retained.get(&5_000).unwrap(), "5000");
    assert_eq!(map.get(&5_000).unwrap(), "5000!");
    for key in 0..10_000 {
        map.remove(&key);
    }
    assert!(map.is_empty());
    assert_eq!(retained.len(), 10_000);
}

#[test]
fn inline_header_footprints_remove_only_actual_header_arcs() {
    use crate::identity::data::{KindId, RelationId};
    use crate::storage::partition::AdjacencySet;
    use crate::storage::substrate::{
        visit_inline_value, StorageAllocationObservation, StorageAllocationVisitor,
    };
    type Header = SharedMap<RelationId, ()>;
    fn header_arc<T>() -> u64 {
        std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<T>())
            .unwrap()
            .0
            .pad_to_align()
            .size() as u64
    }
    #[derive(Default)]
    struct Inventory(Vec<StorageAllocationObservation>);
    impl StorageAllocationVisitor for Inventory {
        fn visit(&mut self, value: StorageAllocationObservation) -> bool {
            self.0.push(value);
            true
        }
    }
    fn inventory<K: Ord + Copy, V: StoredValue>(map: &SharedMap<K, V>) -> Inventory {
        let mut found = Inventory::default();
        map.visit_allocations(true, &mut found, &mut visit_inline_value);
        found
    }
    let old_kind = Arc::<MapNode<KindId, Arc<Header>>>::layout_bytes();
    let new_kind = Arc::<MapNode<KindId, Header>>::layout_bytes();
    let kind_header = header_arc::<Header>();
    assert_eq!(kind_header, Arc::<Header>::layout_bytes());
    let old_kind_map: SharedMap<KindId, Arc<Header>> =
        [(KindId(7), Header::default())].into_iter().collect();
    let new_kind_map: SharedMap<KindId, Header> =
        [(KindId(7), Header::default())].into_iter().collect();
    let old = inventory(&old_kind_map);
    let new = inventory(&new_kind_map);
    assert_eq!(old.0.len(), 2);
    assert_eq!(new.0.len(), 1);
    assert_eq!(
        old.0.iter().map(|owner| owner.bytes).sum::<u64>(),
        old_kind + kind_header
    );
    assert_eq!(new.0.iter().map(|owner| owner.bytes).sum::<u64>(), new_kind);
    assert!(new_kind < old_kind + kind_header);
    let old_slot = Arc::<MapNode<usize, Arc<AdjacencySet>>>::layout_bytes();
    let new_slot = Arc::<MapNode<usize, AdjacencySet>>::layout_bytes();
    let slot_header = header_arc::<AdjacencySet>();
    assert_eq!(slot_header, Arc::<AdjacencySet>::layout_bytes());
    let policy = crate::config::data::AdjacencyPolicy {
        backend: crate::config::data::AdjacencyBackend::InlineSmallDegreeAdjacency,
        small_degree_inline_capacity: 4,
    };
    let old_slot_map: SharedMap<usize, Arc<AdjacencySet>> =
        [(11, AdjacencySet::new(&policy))].into_iter().collect();
    let new_slot_map: SharedMap<usize, AdjacencySet> =
        [(11, AdjacencySet::new(&policy))].into_iter().collect();
    let old = inventory(&old_slot_map);
    let new = inventory(&new_slot_map);
    assert_eq!(old.0.len(), 2);
    assert_eq!(new.0.len(), 1);
    assert_eq!(
        old.0.iter().map(|owner| owner.bytes).sum::<u64>(),
        old_slot + slot_header
    );
    assert_eq!(new.0.iter().map(|owner| owner.bytes).sum::<u64>(), new_slot);
    assert!(new_slot < old_slot + slot_header);
    eprintln!("kind header: old node {old_kind} + header Arc {kind_header}, inline node {new_kind}; adjacency header: old node {old_slot} + header Arc {slot_header}, inline node {new_slot}");
}
