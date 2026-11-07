//! Ordered support counts for the oldest version held by active snapshots.
use std::collections::BTreeMap;

#[derive(Debug)]
pub(super) struct VersionCounts<V> {
    counts: BTreeMap<V, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oldest_support_matches_a_scan_through_duplicate_retains_and_releases() {
        for seed in 1_u64..=8 {
            let mut counts = VersionCounts::new();
            let mut retained = Vec::new();
            let mut random = seed;
            for _ in 0..512 {
                random = random.wrapping_mul(6364136223846793005_u64).wrapping_add(1);
                let version = (random >> 32) % 32;
                if random & 1 == 0 {
                    counts.insert(version);
                    retained.push(version);
                } else {
                    counts.remove(&version);
                    if let Some(index) = retained.iter().position(|value| *value == version) {
                        retained.swap_remove(index);
                    }
                }
                assert_eq!(counts.oldest().copied(), retained.iter().copied().min());
            }
            for version in retained {
                counts.remove(&version);
            }
            assert!(counts.oldest().is_none());
        }
    }

    mod cost {
        use super::*;
        use std::{cell::Cell, cmp::Ordering, rc::Rc};

        #[derive(Clone)]
        struct TrackedVersion(u64, Rc<Cell<u64>>);
        impl PartialEq for TrackedVersion {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }
        impl Eq for TrackedVersion {}
        impl PartialOrd for TrackedVersion {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for TrackedVersion {
            fn cmp(&self, other: &Self) -> Ordering {
                self.1.set(self.1.get() + 1);
                self.0.cmp(&other.0)
            }
        }

        fn selection_work(size: u64) -> u64 {
            let comparisons = Rc::new(Cell::new(0));
            let mut counts = VersionCounts::new();
            for version in 0..size {
                counts.insert(TrackedVersion(version, Rc::clone(&comparisons)));
            }
            comparisons.set(0);
            let selected = counts.oldest().unwrap().0;
            let work = comparisons.get();
            assert_eq!(selected, 0);
            work.max(1)
        }

        #[test]
        fn oldest_selection_does_not_compare_every_retained_version() {
            let small = selection_work(512);
            let large = selection_work(1024);
            assert!(
                large * 2 < small * 3,
                "oldest lookup must stay below linear growth: {small} -> {large}"
            );
        }
    }
}
impl<V: Ord> VersionCounts<V> {
    pub(super) fn new() -> Self {
        Self {
            counts: BTreeMap::new(),
        }
    }
    pub(super) fn insert(&mut self, version: V) {
        *self.counts.entry(version).or_default() += 1;
    }
    pub(super) fn remove(&mut self, version: &V) {
        if let Some(count) = self.counts.get_mut(version) {
            if *count == 1 {
                self.counts.remove(version);
            } else {
                *count -= 1;
            }
        }
    }
    pub(super) fn oldest(&self) -> Option<&V> {
        self.counts.first_key_value().map(|(version, _)| version)
    }
}
