use super::*;
use crate::storage::substrate::{SharedColumn, SharedMap};
use std::collections::BTreeMap;

#[derive(Default)]
struct Inventory(BTreeMap<u64, u64>);
impl StorageAllocationVisitor for Inventory {
    fn visit(&mut self, allocation: StorageAllocationObservation) -> bool {
        if let Some(previous) = self.0.insert(allocation.id, allocation.bytes) {
            assert_eq!(previous, allocation.bytes);
            false
        } else {
            true
        }
    }
}

#[test]
fn column_owner_walk_deduplicates_retained_pages_and_values() {
    let column: SharedColumn<u64> = (0..1024).collect();
    let mut original = Inventory::default();
    column.visit_allocations(true, &mut original, &mut visit_inline_value);
    assert_eq!(original.0.values().sum::<u64>(), column.allocation_bytes());
    let mut changed = column.clone();
    changed.set(500, 9000);
    let mut current = Inventory::default();
    changed.visit_allocations(true, &mut current, &mut visit_inline_value);
    // A full 32-page binary column has five branch levels plus its leaf page;
    // changing one value detaches those six nodes and that one value only.
    assert_eq!(
        current
            .0
            .keys()
            .filter(|id| !original.0.contains_key(id))
            .count(),
        7
    );
    let mut combined = Inventory::default();
    column.visit_allocations(true, &mut combined, &mut visit_inline_value);
    changed.visit_allocations(true, &mut combined, &mut visit_inline_value);
    assert_eq!(combined.0.len(), original.0.len() + 7);
    assert_eq!(column[500], 500);
}

#[test]
fn map_walk_matches_owner_bytes_and_missing_mutations_reuse_every_owner() {
    let map: SharedMap<u64, u64> = (0..1024).map(|key| (key, key)).collect();
    let mut original = Inventory::default();
    map.visit_allocations(true, &mut original, &mut visit_inline_value);
    assert_eq!(original.0.values().sum::<u64>(), map.allocation_bytes());
    let mut changed = map.clone();
    assert!(changed.get_mut(&9000).is_none());
    assert_eq!(changed.remove(&9000), (false, 0));
    let mut after_missing = Inventory::default();
    changed.visit_allocations(true, &mut after_missing, &mut visit_inline_value);
    assert_eq!(original.0, after_missing.0);
    *changed.get_mut(&500).unwrap() = 9000;
    let mut combined = Inventory::default();
    map.visit_allocations(true, &mut combined, &mut visit_inline_value);
    changed.visit_allocations(true, &mut combined, &mut visit_inline_value);
    assert!(combined.0.len() < original.0.len() + 20);
    assert_eq!(map.get(&500), Some(&500));
}

fn visit_delta_inline<T>(
    value: &T,
    _previous: Option<&T>,
    allocation: StorageAllocationObservation,
    visitor: &mut dyn StorageAllocationVisitor,
) {
    visit_inline_value(value, allocation, visitor);
}

#[test]
fn column_delta_walk_matches_independent_full_owner_set_subtraction() {
    for len in [0, 1, 31, 32, 33, 63, 64, 65, 1024, 1025] {
        let mut column: SharedColumn<u64> = (0..len).map(|i| i as u64).collect();
        let previous = column.clone();
        for index in [0, 31, 32, 63, 64, 1023, 1024]
            .into_iter()
            .filter(|&i| i < len)
        {
            column.set(index, 9000);
        }
        for _ in 0..33 {
            column.push(42);
        }
        let mut before = Inventory::default();
        previous.visit_allocations(false, &mut before, &mut visit_inline_value);
        let mut after = Inventory::default();
        column.visit_allocations(false, &mut after, &mut visit_inline_value);
        after.0.retain(|id, _| !before.0.contains_key(id));
        let mut delta = Inventory::default();
        column.visit_new_allocations(&previous, &mut delta, &mut visit_delta_inline);
        assert_eq!(delta.0, after.0);
    }
}

proptest::proptest! {
    #[test]
    fn map_delta_walk_matches_independent_full_owner_subtraction_through_rotations(actions in proptest::collection::vec((0u64..80, proptest::bool::ANY), 0..100)) {
        let mut map = SharedMap::<u64, u64>::new();
        for (key, insert) in actions {
            let previous = map.clone();
            if insert { map.insert(key, key + 9000); }
            else { map.remove(&key); }
            let mut before = Inventory::default();
            previous.visit_allocations(false, &mut before, &mut visit_inline_value);
            let mut after = Inventory::default();
            map.visit_allocations(false, &mut after, &mut visit_inline_value);
            after.0.retain(|id, _| !before.0.contains_key(id));
            let mut delta = Inventory::default();
            map.visit_new_allocations(&previous, &mut delta, &mut visit_delta_inline);
            proptest::prop_assert_eq!(delta.0, after.0);
        }
    }
}
