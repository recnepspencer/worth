//! Actual pinned im allocations of a successor edited beside its shared
//! predecessor, isolated from concurrent test threads, against what the
//! production edit sequence admitted on the real request meter first.

use im::{OrdMap, OrdSet};
use stats_alloc::{Region, Stats, INSTRUMENTED_SYSTEM};
use worth_relational::facade::mvcc::CompanionPreflightBudget;

use super::super::super::InvalidationEditAdmission;
use super::PreparedTreeEdits;

const PROBE_ENV: &str = "WORTH_QUERY_TREE_EDIT_ALLOCATION_PROBE";

#[test]
fn edited_successors_allocate_no_more_than_their_sequence_admitted() {
    // libtest names omit the crate component of module_path!().
    let filter = concat!(module_path!(), "::isolated_tree_edit_allocation_probe")
        .split_once("::")
        .unwrap()
        .1;
    let output = crate::process_deadline::output(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", filter, "--test-threads=1", "--nocapture"])
            .env(PROBE_ENV, "1"),
    )
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
fn isolated_tree_edit_allocation_probe() {
    if std::env::var_os(PROBE_ENV).is_none() {
        return;
    }
    // Initialize allocator/test-thread plumbing outside the measured regions.
    drop(OrdMap::<u64, u64>::new());
    for population in [0, 1, 31, 32, 63, 64, 2047, 2048, 5000] {
        for removes in [false, true] {
            edited_map(population, removes);
            edited_set(population, removes);
        }
    }
}

/// Predecessor keys are even; edits spread over the whole key range, so a
/// run mixes new keys, replacements and, when asked, removals in every part
/// of the tree. Every edit is admitted before it runs.
fn edits(population: usize) -> impl Iterator<Item = (usize, u64)> {
    let range = 2 * (population + edit_count(population)) as u64 + 1;
    (0..edit_count(population)).map(move |step| (step, (step as u64 * 7919) % range))
}

fn edit_count(population: usize) -> usize {
    2 * population.max(64)
}

fn removes_at(step: usize, removes: bool) -> bool {
    removes && step % 7 == 3
}

fn meter() -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: None,
        maximum_preparation_bytes: None,
    })
}

fn live(stats: Stats) -> usize {
    stats
        .bytes_allocated
        .checked_sub(stats.bytes_deallocated)
        .unwrap()
}

fn edited_map(population: usize, removes: bool) {
    let predecessor: OrdMap<u64, u64> = (0..population as u64).map(|k| (2 * k, k)).collect();
    let mut successor = predecessor.clone();
    let mut meter = meter();
    let mut sequence = PreparedTreeEdits::<u64, u64>::new();
    let region = Region::new(&INSTRUMENTED_SYSTEM);
    for (step, key) in edits(population) {
        if removes_at(step, removes) {
            sequence.remove(successor.len(), &mut meter).unwrap();
            successor.remove(&key);
        } else {
            sequence.insert(successor.len(), &mut meter).unwrap();
            successor.insert(key, key);
        }
        assert!(region.change().bytes_allocated as u64 <= meter.charged_bytes());
    }
    assert_eq!(predecessor.len(), population, "predecessor stays intact");
    assert!((0..population as u64).all(|k| predecessor.get(&(2 * k)) == Some(&k)));
    drop(successor);
    assert_eq!(
        live(region.change()),
        0,
        "predecessor owns the shared nodes"
    );
}

fn edited_set(population: usize, removes: bool) {
    let predecessor: OrdSet<u64> = (0..population as u64).map(|k| 2 * k).collect();
    let mut successor = predecessor.clone();
    let mut meter = meter();
    let mut sequence = PreparedTreeEdits::<u64, ()>::new();
    let region = Region::new(&INSTRUMENTED_SYSTEM);
    for (step, key) in edits(population) {
        if removes_at(step, removes) {
            sequence.remove(successor.len(), &mut meter).unwrap();
            successor.remove(&key);
        } else {
            sequence.insert(successor.len(), &mut meter).unwrap();
            successor.insert(key);
        }
        assert!(region.change().bytes_allocated as u64 <= meter.charged_bytes());
    }
    assert_eq!(predecessor.len(), population, "predecessor stays intact");
    assert!((0..population as u64).all(|k| predecessor.contains(&(2 * k))));
    drop(successor);
    assert_eq!(
        live(region.change()),
        0,
        "predecessor owns the shared nodes"
    );
}
