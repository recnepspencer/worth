//! Observe actual std B-tree comparisons independently of the admission formula.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use super::tree_lookup_work;

static COMPARISONS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Eq, PartialEq)]
struct CountedKey(usize);

impl Ord for CountedKey {
    fn cmp(&self, other: &Self) -> Ordering {
        COMPARISONS.fetch_add(1, AtomicOrdering::Relaxed);
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for CountedKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn assert_bounded_search(tree: &BTreeMap<CountedKey, ()>, upper_key: usize) {
    let admitted = tree_lookup_work::<CountedKey>(tree.len()).unwrap() as usize;
    for key in 0..=upper_key {
        COMPARISONS.store(0, AtomicOrdering::Relaxed);
        let _ = tree.get(&CountedKey(key));
        let observed = COMPARISONS.load(AtomicOrdering::Relaxed);
        assert!(
            observed <= admitted,
            "std B-tree compared {observed} keys with {} entries and {admitted} admitted visits",
            tree.len(),
        );
    }
}

#[test]
fn structural_lookup_work_covers_real_root_splits_and_deletions() {
    for entries in [0, 1, 5, 6, 10, 11, 12, 70, 71, 72, 430, 431, 432, 1_000] {
        let natural: Vec<_> = (0..entries).collect();
        let mut reverse = natural.clone();
        reverse.reverse();
        let mut mixed = natural.clone();
        mixed.sort_by_key(|key| (*key as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        for insertion in [natural, reverse, mixed] {
            let mut tree = BTreeMap::new();
            for key in insertion {
                tree.insert(CountedKey(key), ());
            }
            assert_bounded_search(&tree, entries);
            for key in (0..entries).step_by(3) {
                tree.remove(&CountedKey(key));
            }
            assert_bounded_search(&tree, entries);
        }
    }
    assert_eq!(tree_lookup_work::<CountedKey>(0), Some(1));
    assert_eq!(tree_lookup_work::<CountedKey>(6), Some(6));
}
