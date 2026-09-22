use super::*;
use proptest::prelude::*;
use std::collections::BTreeMap;

proptest! {
    #[test]
    fn persistent_edits_match_an_independent_ordered_map(actions in prop::collection::vec((0u16..200, any::<bool>(), any::<u64>()), 0..500)) {
        let mut map = SharedMap::new();
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
            assert_balanced(&map.root);
            assert_balanced(&retained.root);
        }
    }
}

fn assert_balanced<K: Ord + Copy, V: Clone>(
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
    let mut map: SharedMap<_, _> = (0..1024)
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
    let mut map: SharedMap<_, _> = (0..10_000).map(|key| (key, key.to_string())).collect();
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
