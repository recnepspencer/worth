//! Pinned-tree depth evidence and actual COW insertions, with old roots held.
use super::{ordered_edit_bytes, ordered_insertion_bytes, ordered_node_bytes};
use im::OrdMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[test]
fn insertion_forecast_respects_sparse_stable_height_and_root_split_overlap() {
    let node = ordered_node_bytes::<usize, ()>().unwrap();
    // A root may have one key; every nonroot has at least 31. These are the
    // minimal stable trees, independently counting root plus32-child levels.
    for (entries, nodes) in [
        (0, 5),
        (1, 5),
        (62, 5),
        (63, 7),
        (64, 7),
        (2046, 7),
        (2047, 9),
        (2048, 9),
    ] {
        assert_eq!(
            ordered_insertion_bytes::<usize, ()>(entries),
            Some(node * nodes)
        );
        assert!(ordered_edit_bytes::<usize, ()>(entries).unwrap() >= node * nodes);
    }
    assert_eq!(ordered_insertion_bytes::<usize, ()>(1000), Some(node * 7));
    assert_eq!(ordered_insertion_bytes::<usize, ()>(usize::MAX), None);
    assert_eq!(
        ordered_insertion_bytes::<[u8; 4096], [u8; 4096]>(usize::MAX),
        None
    );
}

struct CopiedValue {
    ordinal: usize,
    copies: Arc<AtomicUsize>,
}
impl Clone for CopiedValue {
    fn clone(&self) -> Self {
        self.copies.fetch_add(1, Ordering::Relaxed);
        Self {
            ordinal: self.ordinal,
            copies: Arc::clone(&self.copies),
        }
    }
}

#[test]
fn actual_pinned_insertions_fit_copy_forecast_without_displacing_prior_roots() {
    for population in [0, 1, 62, 63, 64, 2047, 2048, 65_536] {
        for order in 0..3 {
            let copies = Arc::new(AtomicUsize::new(0));
            let mut current = OrdMap::new();
            for ordinal in 0..population {
                let key = match order {
                    0 => ordinal,
                    1 => population - ordinal - 1,
                    _ if ordinal % 2 == 0 => ordinal / 2,
                    _ => population - ordinal / 2 - 1,
                };
                current.insert(
                    key,
                    CopiedValue {
                        ordinal: key,
                        copies: Arc::clone(&copies),
                    },
                );
            }
            let prior = current.clone();
            let quote = ordered_insertion_bytes::<usize, CopiedValue>(population).unwrap();
            let node = ordered_node_bytes::<usize, CopiedValue>().unwrap();
            copies.store(0, Ordering::Relaxed);
            current.insert(
                population,
                CopiedValue {
                    ordinal: population,
                    copies: Arc::clone(&copies),
                },
            );
            // The source-backed node/split bound additionally covers allocated
            // split nodes; this independent counter observes actual payload COW.
            assert!(copies.load(Ordering::Relaxed) as u64 <= (quote / node) * 64);
            assert_eq!(prior.len(), population);
            assert!(prior.get(&population).is_none());
            assert_eq!(current.get(&population).unwrap().ordinal, population);
            for key in [0, population / 2, population.saturating_sub(1)] {
                if let Some(value) = prior.get(&key) {
                    assert_eq!(value.ordinal, key);
                }
            }
            // Replacement of a present key uses the same insertion-only owner.
            if population > 0 {
                let replacement_prior = current.clone();
                copies.store(0, Ordering::Relaxed);
                current.insert(
                    0,
                    CopiedValue {
                        ordinal: population + 1,
                        copies: Arc::clone(&copies),
                    },
                );
                let replacement =
                    ordered_insertion_bytes::<usize, CopiedValue>(population + 1).unwrap();
                assert!(copies.load(Ordering::Relaxed) as u64 <= (replacement / node) * 64);
                assert_eq!(prior.get(&0).unwrap().ordinal, 0);
                assert_eq!(replacement_prior.get(&0).unwrap().ordinal, 0);
                assert_eq!(current.get(&0).unwrap().ordinal, population + 1);
            }
        }
    }
}
