//! Structural allocation courts: no history, payload, or preparation slack.
use super::measure;
use crate::data::retained_storage::{ordered_batch_growth_charge, ordered_forest_growth_charge};

#[test]
fn ordered_batch_bound_covers_a_three_level_root_split() {
    // im 15.1 ascending inserts: the first root splits at 65, then the right
    // spine fills every 33 inserts per level. The third root split is at
    // 65 + 64*33 + 64*33^2 = 71_873 entries. Use a window covering it, with each original root
    // shared. A single edit's charge must cover the entire cascade alone.
    let mut map = im::OrdMap::<u64, u64>::new();
    for key in 0..70_000 {
        map.insert(key, key);
    }
    let mut largest = 0;
    for key in 70_000..73_000 {
        let snapshot = map.clone();
        let forecast = ordered_batch_growth_charge::<u64, u64>(map.len() + 1, 1)
            .unwrap()
            .bytes();
        let (updated, peak) = measure(|| {
            map.insert(key, key);
            map
        });
        let peak = peak.unwrap() as u64;
        assert!(peak <= forecast, "key {key}: {peak} > {forecast}");
        largest = largest.max(peak);
        map = updated;
        assert_eq!(snapshot.len() + 1, map.len());
    }
    // A cascade allocates more than a plain three-node copy-on-write path.
    // This structural witness also detects a fixture that missed the split.
    assert!(
        largest
            > crate::data::retained_storage::ordered_index_charge::<u64, u64>(0)
                .unwrap()
                .bytes()
                * 5,
        "three-level cascade was not exercised: {largest}"
    );
}

#[test]
fn diagnostics_overlay_bound_covers_shared_retirement_and_readmission_at_scale() {
    use crate::data::persistent_ord_map::PersistentOrdMap;
    let mut base = PersistentOrdMap::<u64, u64>::new();
    for key in 0..16_384 {
        base.insert(key, key);
    }
    let mut overlay = base.fork_persistent();
    for key in (0..16_384).step_by(2) {
        overlay.insert(key, key + 1);
    }
    let snapshot = overlay.fork_persistent();
    for retire in [true, false] {
        let forecast = overlay
            .batch_edit_structure_growth_bound(8_192)
            .unwrap()
            .bytes();
        let (updated, peak) = measure(|| {
            for key in (1..16_384).step_by(2) {
                if retire {
                    overlay.remove(&key);
                } else {
                    overlay.insert(key, key);
                }
            }
            overlay
        });
        assert!(
            peak.unwrap() as u64 <= forecast,
            "retire={retire}: {peak:?} > {forecast}"
        );
        overlay = updated;
    }
    assert_eq!(overlay.len(), snapshot.len());
    for key in 0..16_384 {
        assert_eq!(overlay.get(&key), snapshot.get(&key));
    }
}

#[test]
fn ordered_batch_bound_alone_covers_shared_three_level_trees() {
    // More than 64 + 65*64 entries requires at least three im 15.1 levels.
    for count in [1, 32, 300, 8_192] {
        let mut map: im::OrdMap<u64, u64> = (0_u64..8_192).map(|key| (key, key)).collect();
        let snapshot = map.clone();
        let forecast = ordered_batch_growth_charge::<u64, u64>(8_192 + count, count)
            .unwrap()
            .bytes();
        let (map, peak) = measure(|| {
            for key in 8_192..8_192 + count as u64 {
                // Retained publication keeps the previous staged root through
                // each edit; all older intermediate roots must then be dropped.
                let previous = map.clone();
                map.insert(key, key);
                std::hint::black_box(previous);
            }
            map
        });
        assert!(
            peak.unwrap() as u64 <= forecast,
            "{count} edits: {peak:?} > {forecast}"
        );
        assert_eq!(map.len(), 8_192 + count);
        assert_eq!(snapshot.len(), 8_192);
    }
}

#[test]
fn ordered_batch_bound_alone_covers_retirement_and_readmission() {
    let mut map: im::OrdMap<u64, u64> = (0_u64..16_384).map(|key| (key, key)).collect();
    let snapshot = map.clone();
    for retire in [true, false] {
        let forecast = ordered_batch_growth_charge::<u64, u64>(16_384, 8_192)
            .unwrap()
            .bytes();
        let (updated, peak) = measure(|| {
            for key in (0..16_384).step_by(2) {
                let previous = map.clone();
                if retire {
                    map.remove(&key);
                } else {
                    map.insert(key, key);
                }
                std::hint::black_box(previous);
            }
            map
        });
        assert!(
            peak.unwrap() as u64 <= forecast,
            "retire={retire}: {peak:?} > {forecast}"
        );
        map = updated;
    }
    assert_eq!(map, snapshot);
}

#[test]
fn history_forest_bound_alone_covers_distinct_and_shared_histories() {
    for histories in [1, 8_192] {
        let mut forest = vec![im::OrdMap::<u64, u64>::new(); histories];
        for key in 0..8_192 {
            forest[key % histories].insert(key as u64, key as u64);
        }
        let snapshot = forest.clone();
        let forecast = ordered_forest_growth_charge::<u64, u64>(16_384, 16_384)
            .unwrap()
            .bytes();
        let (forest, peak) = measure(|| {
            for key in 8_192..16_384 {
                let tree = &mut forest[key % histories];
                let previous = tree.clone();
                tree.insert(key as u64, key as u64);
                std::hint::black_box(previous);
            }
            for key in 0..8_192 {
                let tree = &mut forest[key % histories];
                let previous = tree.clone();
                tree.remove(&(key as u64));
                std::hint::black_box(previous);
            }
            forest
        });
        assert!(
            peak.unwrap() as u64 <= forecast,
            "{histories} histories: {peak:?} > {forecast}"
        );
        assert_eq!(forest.iter().map(im::OrdMap::len).sum::<usize>(), 8_192);
        assert_eq!(snapshot.iter().map(im::OrdMap::len).sum::<usize>(), 8_192);
    }
}
