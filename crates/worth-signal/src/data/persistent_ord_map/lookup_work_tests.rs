//! Compare the structural search allowance with real installed tree searches.
use std::cell::Cell;
use std::cmp::Ordering;
use std::rc::Rc;

use super::PersistentOrdMap;

#[derive(Clone)]
struct ObservedKey {
    value: usize,
    comparisons: Rc<Cell<usize>>,
}

impl PartialEq for ObservedKey {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
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
        self.comparisons.set(self.comparisons.get() + 1);
        self.value.cmp(&other.value)
    }
}

#[test]
fn search_allowance_covers_empty_flat_overlay_and_retirement_paths() {
    for size in [
        0, 1, 2, 10, 11, 12, 31, 62, 63, 64, 65, 70, 71, 72, 1_000, 2_046, 2_047, 2_048, 4_225,
    ] {
        let comparisons = Rc::new(Cell::new(0));
        let key = |value| ObservedKey {
            value,
            comparisons: comparisons.clone(),
        };
        let mut flat = PersistentOrdMap::new();
        for value in 0..size {
            flat.insert(key(value * 2), value);
        }
        check_searches(&flat, size * 2 + 2, &comparisons);
        let mut overlay = flat.fork_persistent();
        for value in (0..size).step_by(3) {
            overlay.remove(&key(value * 2));
        }
        for value in (0..size).step_by(5) {
            overlay.insert(key(value * 2 + 1), value);
        }
        check_searches(&overlay, size * 2 + 2, &comparisons);
        // An empty inherited base keeps every insertion in the real im tree.
        // Fixture size is therefore its physical entry count, including the
        // minimum-fill boundaries and a forced third-level tree at 4,225.
        let mut empty = PersistentOrdMap::new();
        let mut dense_overlay = empty.fork_persistent();
        for value in 0..size {
            dense_overlay.insert(key(value * 2), value);
        }
        check_searches(&dense_overlay, size * 2 + 2, &comparisons);
        for value in (0..size).step_by(3) {
            dense_overlay.remove(&key(value * 2));
        }
        check_searches(&dense_overlay, size * 2 + 2, &comparisons);
        // Retired keys can re-enter the base through an overlay and split an
        // interval. Queries must still cover both exact and predecessor paths.
        for value in (0..size).step_by(6) {
            overlay.insert(key(value * 2), value);
        }
        check_searches(&overlay, size * 2 + 2, &comparisons);
    }
}

fn check_searches(
    map: &PersistentOrdMap<ObservedKey, usize>,
    queries: usize,
    comparisons: &Rc<Cell<usize>>,
) {
    for value in 0..queries {
        let query = ObservedKey {
            value,
            comparisons: comparisons.clone(),
        };
        comparisons.set(0);
        let _ = map.get(&query);
        assert!(
            comparisons.get() <= map.lookup_steps(),
            "query {value}: {} comparisons exceed {} search steps",
            comparisons.get(),
            map.lookup_steps(),
        );
    }
}
