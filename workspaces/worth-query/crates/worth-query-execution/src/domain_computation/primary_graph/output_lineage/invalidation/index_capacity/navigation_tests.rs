use std::{
    cmp::Ordering,
    sync::{
        atomic::{AtomicU64, Ordering as AtomicOrdering},
        Arc,
    },
};

use im::OrdMap;

use super::{ordered_navigation_work, ordered_removal_work};

#[derive(Clone)]
struct ObservedKey {
    value: usize,
    comparisons: Arc<AtomicU64>,
}

impl PartialEq for ObservedKey {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for ObservedKey {}

impl PartialOrd for ObservedKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ObservedKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.comparisons.fetch_add(1, AtomicOrdering::Relaxed);
        self.value.cmp(&other.value)
    }
}

fn key(value: usize, comparisons: &Arc<AtomicU64>) -> ObservedKey {
    ObservedKey {
        value,
        comparisons: Arc::clone(comparisons),
    }
}

fn probe(map: &OrdMap<ObservedKey, ()>, population: usize, comparisons: &Arc<AtomicU64>) {
    let admitted = ordered_navigation_work(map.len()).unwrap();
    for ordinal in 0..513 {
        // Present, removed and absent keys, distributed across the whole tree.
        let value = ordinal * (population + 1) / 256;
        comparisons.store(0, AtomicOrdering::Relaxed);
        let _ = map.get(&key(value, comparisons));
        let actual = comparisons.load(AtomicOrdering::Relaxed);
        assert!(
            actual <= admitted,
            "{} entries, key {value}: {actual} comparisons exceeded {admitted}",
            map.len(),
        );
    }
}

#[test]
fn admitted_navigation_covers_actual_im_searches_after_growth_and_deletion() {
    for population in [0, 1, 62, 63, 64, 65, 2046, 2047, 2048, 65_536] {
        for order in 0..3 {
            let comparisons = Arc::new(AtomicU64::new(0));
            let mut map = OrdMap::new();
            for ordinal in 0..population {
                let value = match order {
                    0 => ordinal,
                    1 => population - ordinal - 1,
                    // Interleave the extremes without repeating keys.
                    _ if ordinal % 2 == 0 => ordinal / 2,
                    _ => population - ordinal / 2 - 1,
                };
                let admitted = ordered_navigation_work(map.len()).unwrap();
                comparisons.store(0, AtomicOrdering::Relaxed);
                map.insert(key(value, &comparisons), ());
                assert!(
                    comparisons.load(AtomicOrdering::Relaxed) <= admitted,
                    "insertion at {} entries exceeded {admitted}",
                    map.len() - 1,
                );
            }
            probe(&map, population, &comparisons);
            for ordinal in 0..population {
                let value = if order == 1 {
                    population - ordinal - 1
                } else {
                    ordinal
                };
                let entries = map.len();
                let admitted = ordered_removal_work(entries).unwrap();
                comparisons.store(0, AtomicOrdering::Relaxed);
                map.remove(&key(value, &comparisons));
                assert!(
                    comparisons.load(AtomicOrdering::Relaxed) <= admitted,
                    "removal at {entries} entries exceeded {admitted}",
                );
                if [0, 1, 62, 63, 64, 65, 2046, 2047, 2048].contains(&map.len()) {
                    probe(&map, population, &comparisons);
                }
            }
        }
    }
}

#[test]
fn navigation_charge_does_not_invent_depth_for_a_small_index() {
    // A small root has bounded binary-search cost regardless of cardinality;
    // the prior bit-length height wrongly multiplied that work by five.
    assert!(ordered_navigation_work(31).unwrap() <= 16);
    assert!(ordered_navigation_work(usize::MAX).is_some());
}
