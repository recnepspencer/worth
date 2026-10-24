//! The im OrdMap charges describe im 15.1's node layout, so the dependency is
//! pinned. These courts measure the installed map and fail when a different
//! im allocates more than the charges admit.
use std::env;
use std::process::Command;

use im::OrdMap;
use stats_alloc::{Region, Stats, INSTRUMENTED_SYSTEM};

use super::{ordered_edit_growth_charge, ordered_index_charge};

const TEST_NAME: &str = "data::retained_storage::ordered_index::tests::installed_im_ordmap_allocates_within_its_charges";
const SIZES: [usize; 8] = [0, 1, 31, 64, 65, 1_000, 4_097, 20_000];

fn live(stats: Stats) -> u64 {
    let live = stats.bytes_allocated as i128 - stats.bytes_deallocated as i128
        + stats.bytes_reallocated as i128;
    u64::try_from(live.max(0)).expect("live bytes fit u64")
}

/// Sequential keys fill nodes in split order; the multiplicative sequence
/// scatters them so splits land mid-node.
fn keys(entries: usize, scattered: bool) -> impl Iterator<Item = u64> {
    (0..entries as u64).map(move |key| {
        if scattered {
            key.wrapping_mul(0x9e37_79b9_7f4a_7c15)
        } else {
            key
        }
    })
}

fn built(entries: usize, scattered: bool) -> (OrdMap<u64, u64>, u64) {
    let region = Region::new(&INSTRUMENTED_SYSTEM);
    let mut map = OrdMap::new();
    for key in keys(entries, scattered) {
        map.insert(key, key);
    }
    let bytes = live(region.change());
    (map, bytes)
}

fn edit_bytes(map: &OrdMap<u64, u64>, edit: impl FnOnce(&mut OrdMap<u64, u64>)) -> u64 {
    // The clone shares every node, so the edit copies its whole path.
    let mut shared = map.clone();
    let region = Region::new(&INSTRUMENTED_SYSTEM);
    edit(&mut shared);
    let allocated = region.change().bytes_allocated as u64;
    drop(shared);
    allocated
}

#[test]
fn installed_im_ordmap_allocates_within_its_charges() {
    const CHILD_PROCESS: &str = "WORTH_SIGNAL_ORDERED_INDEX_CHILD";
    if env::var_os(CHILD_PROCESS).is_none() {
        let output = crate::process_deadline::output(
            Command::new(env::current_exe().expect("test executable resolves"))
                .arg("--exact")
                .arg(TEST_NAME)
                .arg("--show-output")
                .arg("--test-threads=1")
                .env(CHILD_PROCESS, "1"),
        )
        .expect("isolated ordered-index probe starts");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "isolated ordered-index probe failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
        assert!(
            stdout.contains(&format!("test {TEST_NAME} ... ok")),
            "isolated ordered-index probe selected no exact test\nstdout:\n{stdout}"
        );
        println!("{stdout}");
        return;
    }

    for entries in SIZES {
        let charge = ordered_index_charge::<u64, u64>(entries).unwrap().bytes();
        let growth = ordered_edit_growth_charge::<u64, u64>(entries)
            .unwrap()
            .bytes();
        for scattered in [false, true] {
            let (map, retained) = built(entries, scattered);
            assert_eq!(map.len(), entries);
            assert!(
                retained <= charge,
                "{entries} entries (scattered {scattered}) retain {retained} bytes, charged {charge}"
            );
            let inserted = edit_bytes(&map, |map| {
                map.insert(u64::MAX, 0);
            });
            assert!(
                inserted <= growth,
                "insert into {entries} entries allocated {inserted} bytes, charged {growth}"
            );
            if let Some(key) = keys(entries, scattered).nth(entries / 2) {
                let removed = edit_bytes(&map, |map| {
                    map.remove(&key);
                });
                assert!(
                    removed <= growth,
                    "remove from {entries} entries allocated {removed} bytes, charged {growth}"
                );
            }
            println!(
                "ordered index {entries} scattered={scattered}: retained {retained}/{charge}, insert {inserted}/{growth}"
            );
        }
    }
}
