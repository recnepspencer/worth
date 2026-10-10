//! Actual pinned im allocations, isolated from concurrent test threads. Keys
//! have no heap payload: the oracle measures nodes, not caller-owned payloads.

use std::fmt::Debug;

use im::{OrdMap, OrdSet};
use stats_alloc::{Region, Stats, INSTRUMENTED_SYSTEM};

use super::{ordered_edit_bytes, retained_forest_bytes, retained_map_bytes};

const PROBE_ENV: &str = "WORTH_QUERY_INDEX_ALLOCATION_PROBE";

#[test]
fn independent_im_allocations_fit_retained_and_copied_node_bounds() {
    // libtest names omit the crate component of module_path!().
    let filter = concat!(module_path!(), "::isolated_im_allocation_probe")
        .split_once("::")
        .unwrap()
        .1;
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", filter, "--test-threads=1", "--nocapture"])
        .env(PROBE_ENV, "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "allocation probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
}

#[test]
fn isolated_im_allocation_probe() {
    if std::env::var_os(PROBE_ENV).is_none() {
        return;
    }
    // Initialize allocator/test-thread plumbing outside the measured regions.
    drop(OrdMap::<u64, u64>::new());
    for population in [0, 1, 31, 32, 62, 63, 64, 65, 2046, 2047, 2048, 70_000] {
        for order in 0..3 {
            if population > 2048 && order != 0 {
                continue;
            }
            whole_map::<u64>(population, order);
            whole_set::<u64>(population, order);
            whole_map::<Aligned>(population, order);
            whole_set::<Aligned>(population, order);
            copied_paths::<u64>(population, order);
            copied_paths::<Aligned>(population, order);
        }
    }
    forest_maps::<u64>();
    forest_sets::<u64>();
    forest_maps::<Aligned>();
    forest_sets::<Aligned>();
    // Real one-node im indexes fit these finite ceilings. The old per-entry
    // retained forecast and binary-height copied forecast both exceeded them.
    assert!(retained_map_bytes::<u64, u64>(32).unwrap() <= 4096);
    assert!(ordered_edit_bytes::<u64, u64>(31).unwrap() <= 8192);
    assert!(ordered_edit_bytes::<u64, u64>(usize::MAX).is_none());
    assert!(retained_forest_bytes::<u64, u64>(31, usize::MAX).is_none());
}

trait ProbeKey: Copy + Ord + Debug {
    fn key(ordinal: usize) -> Self;
}

impl ProbeKey for u64 {
    fn key(ordinal: usize) -> Self {
        ordinal as u64
    }
}

#[repr(align(64))]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Aligned(u64);

impl ProbeKey for Aligned {
    fn key(ordinal: usize) -> Self {
        Self(ordinal as u64)
    }
}

fn ordered_key(ordinal: usize, population: usize, order: usize) -> usize {
    match order {
        0 => ordinal,
        1 => population - ordinal - 1,
        _ if ordinal.is_multiple_of(2) => ordinal / 2,
        _ => population - ordinal / 2 - 1,
    }
}

fn checkpoint(entries: usize) -> bool {
    matches!(
        entries,
        0 | 1 | 31 | 32 | 62 | 63 | 64 | 65 | 2046 | 2047 | 2048 | 65_535 | 65_536
    )
}

fn live(stats: Stats) -> usize {
    stats
        .bytes_allocated
        .checked_sub(stats.bytes_deallocated)
        .unwrap()
}

fn whole_map<K: ProbeKey>(population: usize, order: usize) {
    let region = Region::new(&INSTRUMENTED_SYSTEM);
    let mut map = OrdMap::new();
    for ordinal in 0..population {
        let key = K::key(ordered_key(ordinal, population, order));
        map.insert(key, key);
    }
    assert!(live(region.change()) as u64 <= retained_map_bytes::<K, K>(map.len()).unwrap());
    for ordinal in 0..population {
        let key = K::key(ordered_key(ordinal, population, order));
        assert_eq!(map.remove(&key), Some(key));
        if checkpoint(map.len()) {
            assert!(live(region.change()) as u64 <= retained_map_bytes::<K, K>(map.len()).unwrap());
        }
    }
    drop(map);
    assert_eq!(
        live(region.change()),
        0,
        "last map root must release its nodes"
    );
}

fn whole_set<K: ProbeKey>(population: usize, order: usize) {
    let region = Region::new(&INSTRUMENTED_SYSTEM);
    let mut set = OrdSet::new();
    for ordinal in 0..population {
        set.insert(K::key(ordered_key(ordinal, population, order)));
    }
    assert!(live(region.change()) as u64 <= retained_map_bytes::<K, ()>(set.len()).unwrap());
    for ordinal in 0..population {
        let key = K::key(ordered_key(ordinal, population, order));
        assert_eq!(set.remove(&key), Some(key));
        if checkpoint(set.len()) {
            assert!(
                live(region.change()) as u64 <= retained_map_bytes::<K, ()>(set.len()).unwrap()
            );
        }
    }
    drop(set);
    assert_eq!(
        live(region.change()),
        0,
        "last set root must release its nodes"
    );
}

fn copied_paths<K: ProbeKey>(population: usize, order: usize) {
    let mut map = OrdMap::new();
    let mut set = OrdSet::new();
    for ordinal in 0..population {
        let key = K::key(ordered_key(ordinal, population, order));
        map.insert(key, key);
        set.insert(key);
    }
    probe_edits(&map, &set, population);
    for ordinal in 0..population {
        let key = K::key(ordered_key(ordinal, population, order));
        map.remove(&key);
        set.remove(&key);
        if checkpoint(map.len()) {
            probe_edits(&map, &set, population);
        }
    }
}

fn probe_edits<K: ProbeKey>(map: &OrdMap<K, K>, set: &OrdSet<K>, population: usize) {
    // Present, deleted and never-present keys across every part of the tree.
    for ordinal in 0..17 {
        let key = K::key(ordinal * (population + 1) / 8);
        for insert in [true, false] {
            let old_value = map.get(&key).copied();
            let mut successor = map.clone();
            let region = Region::new(&INSTRUMENTED_SYSTEM);
            if insert {
                successor.insert(key, key);
            } else {
                successor.remove(&key);
            }
            let allocated = region.change().bytes_allocated;
            assert!(allocated as u64 <= ordered_edit_bytes::<K, K>(map.len()).unwrap());
            assert_eq!(
                map.get(&key).copied(),
                old_value,
                "predecessor stays intact"
            );
            drop(successor);
            assert_eq!(
                live(region.change()),
                0,
                "predecessor owns the shared nodes"
            );

            let old_value = set.contains(&key);
            let mut successor = set.clone();
            let region = Region::new(&INSTRUMENTED_SYSTEM);
            if insert {
                successor.insert(key);
            } else {
                successor.remove(&key);
            }
            let allocated = region.change().bytes_allocated;
            assert!(allocated as u64 <= ordered_edit_bytes::<K, ()>(set.len()).unwrap());
            assert_eq!(set.contains(&key), old_value, "predecessor stays intact");
            drop(successor);
            assert_eq!(
                live(region.change()),
                0,
                "predecessor owns the shared nodes"
            );
        }
    }
}

fn forest_maps<K: ProbeKey>() {
    let sizes = [0, 1, 2, 31, 32, 64, 65, 256];
    let mut forest = Vec::with_capacity(64);
    let region = Region::new(&INSTRUMENTED_SYSTEM);
    let mut entries = 0;
    for ordinal in 0..64 {
        let mut map = OrdMap::new();
        let size = sizes[ordinal % sizes.len()];
        for key in 0..size {
            map.insert(K::key(key), K::key(key));
        }
        entries += size;
        forest.push(map);
    }
    assert!(
        live(region.change()) as u64
            <= retained_forest_bytes::<K, K>(entries, forest.len()).unwrap()
    );
    forest.clear();
    assert_eq!(live(region.change()), 0);
}

fn forest_sets<K: ProbeKey>() {
    let sizes = [0, 1, 2, 31, 32, 64, 65, 256];
    let mut forest = Vec::with_capacity(64);
    let region = Region::new(&INSTRUMENTED_SYSTEM);
    let mut entries = 0;
    for ordinal in 0..64 {
        let mut set = OrdSet::new();
        let size = sizes[ordinal % sizes.len()];
        for key in 0..size {
            set.insert(K::key(key));
        }
        entries += size;
        forest.push(set);
    }
    assert!(
        live(region.change()) as u64
            <= retained_forest_bytes::<K, ()>(entries, forest.len()).unwrap()
    );
    forest.clear();
    assert_eq!(live(region.change()), 0);
}
