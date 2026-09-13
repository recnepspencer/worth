use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::env;
use std::hint::black_box;
use std::process::Command;

use stats_alloc::{Region, INSTRUMENTED_SYSTEM};

use super::{PersistentOrdMap, PersistentOrdMapStorage};

thread_local! {
    static KEY_COMPARISONS: Cell<usize> = const { Cell::new(0) };
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CountedKey(u64);

impl Ord for CountedKey {
    fn cmp(&self, other: &Self) -> Ordering {
        KEY_COMPARISONS.set(KEY_COMPARISONS.get() + 1);
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for CountedKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[test]
fn overlay_only_insert_remove_churn_retains_no_historical_tombstones() {
    for churn_count in [64_u64, 4_096, 65_536] {
        let mut source = PersistentOrdMap::<u64, u64>::new();
        source.insert(0, 7);
        let mut fork = source.fork_persistent();
        assert!(source.ptr_eq(&fork), "scale {churn_count} must share");

        for key in 1..=churn_count {
            assert_eq!(fork.get_mut(&key), None);
            assert_eq!(fork.insert(key, key), None);
            assert_eq!(fork.remove(&key), Some(key));
            assert_eq!(fork.get_mut(&key), None);
        }

        assert_eq!(fork.len(), 1);
        assert_eq!(fork.get(&0), Some(&7));
        let PersistentOrdMapStorage::ForkShared { changes, .. } = &fork.storage else {
            panic!("fork must retain shared storage");
        };
        assert!(
            changes.is_empty(),
            "scale {churn_count} overlay-only churn must erase its delta"
        );
        assert_eq!(source.get(&0), Some(&7));
        assert_eq!(source.len(), 1);
    }
}

#[test]
fn inherited_prefix_retirement_does_not_tax_later_ordered_observation() {
    for retired_count in [64_u64, 4_096, 65_536] {
        let mut source = PersistentOrdMap::new();
        for key in 0..=retired_count {
            source.insert(CountedKey(key), key);
        }
        let mut fork = source.fork_persistent();
        assert!(source.ptr_eq(&fork), "scale {retired_count} must share");

        for key in 0..retired_count {
            assert_eq!(fork.remove(&CountedKey(key)), Some(key));
        }
        assert_eq!(fork.len(), 1);
        assert_eq!(source.len(), retired_count as usize + 1);

        KEY_COMPARISONS.set(0);
        assert_eq!(
            fork.first_key_value().map(|(key, value)| (key.0, *value)),
            Some((retired_count, retired_count))
        );
        let first_access_comparisons = KEY_COMPARISONS.get();
        assert!(
            first_access_comparisons <= 64,
            "scale {retired_count} repeated retired history: {first_access_comparisons} comparisons"
        );
        assert_eq!(source.first_key_value().map(|(key, _)| key.0), Some(0));

        let PersistentOrdMapStorage::ForkShared {
            changes,
            retired_base_intervals,
            ..
        } = &fork.storage
        else {
            panic!("fork must retain shared storage");
        };
        assert!(
            changes.is_empty(),
            "retirement must not retain per-key deltas"
        );
        assert_eq!(
            retired_base_intervals.len(),
            1,
            "contiguous retirement must retain one navigation interval"
        );
    }
}

#[test]
fn out_of_order_base_retirement_keeps_first_deletion_bounded() {
    for retired_count in [64_u64, 4_096, 65_536] {
        let mut destination = PersistentOrdMap::new();
        for key in 0..=retired_count {
            destination.insert(CountedKey(key), key);
        }
        let source = destination.fork_persistent();
        assert!(source.ptr_eq(&destination));

        for key in 1..retired_count {
            assert_eq!(destination.remove(&CountedKey(key)), Some(key));
        }
        KEY_COMPARISONS.set(0);
        assert_eq!(destination.remove(&CountedKey(0)), Some(0));
        let first_deletion_comparisons = KEY_COMPARISONS.get();
        assert!(
            first_deletion_comparisons <= 128,
            "scale {retired_count} first deletion repeated retired history: {first_deletion_comparisons} comparisons"
        );
        assert_eq!(
            destination
                .first_key_value()
                .map(|(key, value)| (key.0, *value)),
            Some((retired_count, retired_count))
        );

        let readmitted = retired_count / 2;
        assert_eq!(destination.insert(CountedKey(readmitted), readmitted), None);
        assert_eq!(
            destination.first_key_value().map(|(key, _)| key.0),
            Some(readmitted)
        );
        assert_eq!(
            destination.remove(&CountedKey(readmitted)),
            Some(readmitted)
        );
        assert_eq!(
            destination.first_key_value().map(|(key, _)| key.0),
            Some(retired_count)
        );
        assert_eq!(source.first_key_value().map(|(key, _)| key.0), Some(0));
        assert_eq!(source.len(), retired_count as usize + 1);
    }
}

#[test]
fn retiring_temporary_earlier_key_preserves_inherited_frontier_index() {
    let mut destination: PersistentOrdMap<u64, u64> =
        [(10, 10), (20, 20), (30, 30)].into_iter().collect();
    let source = destination.fork_persistent();
    assert!(source.ptr_eq(&destination));

    assert_eq!(destination.remove(&10), Some(10));
    assert_eq!(destination.remove(&20), Some(20));
    assert_eq!(destination.insert(5, 5), None);
    assert_eq!(destination.first_key_value(), Some((&5, &5)));
    assert_eq!(destination.remove(&5), Some(5));

    assert_eq!(destination.first_key_value(), Some((&30, &30)));
    assert_eq!(destination.iter().collect::<Vec<_>>(), vec![(&30, &30)]);
    assert_eq!(source.iter().count(), 3);
    assert_eq!(source.first_key_value(), Some((&10, &10)));
}

const SHARED_LOGICAL_CLONE_CHILD: &str = "WORTH_SIGNAL_SHARED_MAP_CLONE_COST_CHILD";
const SHARED_LOGICAL_CLONE_TEST: &str =
    "data::persistent_ord_map::tests::shared_logical_clone_is_bounded_for_rollback_capture";

fn run_isolated_shared_logical_clone_probe() {
    let output = Command::new(env::current_exe().expect("test executable resolves"))
        .arg("--exact")
        .arg(SHARED_LOGICAL_CLONE_TEST)
        .arg("--nocapture")
        .arg("--test-threads=1")
        .env(SHARED_LOGICAL_CLONE_CHILD, "1")
        .output()
        .expect("isolated logical-clone allocation probe starts");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    print!("{stdout}");
    eprint!("{stderr}");
    assert!(
        output.status.success()
            && stdout.contains(SHARED_LOGICAL_CLONE_TEST)
            && stdout.contains("1 passed; 0 failed"),
        "logical-clone probe did not run exactly once: stdout={stdout:?} stderr={stderr:?}"
    );
}

#[test]
fn shared_logical_clone_is_bounded_for_rollback_capture() {
    if env::var_os(SHARED_LOGICAL_CLONE_CHILD).is_none() {
        run_isolated_shared_logical_clone_probe();
        return;
    }

    let mut samples = Vec::new();
    let mut copy_bytes = None;
    for entry_count in [64_u64, 4_096, 65_536] {
        let mut source: PersistentOrdMap<u64, u64> =
            (0..entry_count).map(|key| (key, key)).collect();
        if entry_count == 65_536 {
            let copy_region = Region::new(&INSTRUMENTED_SYSTEM);
            black_box(source.operational_clone());
            copy_bytes = Some(copy_region.change().bytes_allocated);
        }
        let mut current = source.fork_persistent();
        current.insert(0, entry_count);

        let region = Region::new(&INSTRUMENTED_SYSTEM);
        let baseline = black_box(current.clone());
        let allocation = region.change();
        samples.push((
            entry_count,
            allocation.allocations,
            allocation.bytes_allocated,
        ));

        assert!(current.ptr_eq(&baseline));
        assert_eq!(source.get(&0), Some(&0));
        assert_eq!(current.get(&0), Some(&entry_count));
        assert_eq!(baseline.get(&0), Some(&entry_count));
        assert_eq!(current.len(), entry_count as usize);
    }

    let minimum_calls = samples.iter().map(|(_, calls, _)| *calls).min().unwrap();
    let minimum_bytes = samples.iter().map(|(_, _, bytes)| *bytes).min().unwrap();
    for (entry_count, calls, bytes) in &samples {
        assert!(
            *calls <= minimum_calls + 32,
            "logical-clone calls slope with {entry_count} entries: {calls} vs {minimum_calls}"
        );
        assert!(
            *bytes <= minimum_bytes + 32 * 1_024,
            "logical-clone bytes slope with {entry_count} entries: {bytes} vs {minimum_bytes}"
        );
    }
    assert!(
        copy_bytes.expect("copy sensitivity sample exists")
            > samples.iter().map(|(_, _, bytes)| *bytes).max().unwrap() * 8,
        "probe must distinguish shared logical cloning from reconstructive operational cloning"
    );
}

#[test]
fn forked_overlay_matches_btree_model_across_mixed_state_transitions() {
    for initial_seed in 1_u64..=8 {
        let mut random = initial_seed;
        let mut siblings = vec![(PersistentOrdMap::new(), BTreeMap::new())];
        for _ in 0..512 {
            random = random
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let source = random as usize % siblings.len();
            let key = random.rotate_left(17) % 128;
            let value = random.rotate_left(31);
            match random.rotate_left(7) % 6 {
                0 => assert_eq!(
                    siblings[source].0.insert(key, value),
                    siblings[source].1.insert(key, value)
                ),
                1 => assert_eq!(
                    siblings[source].0.remove(&key),
                    siblings[source].1.remove(&key)
                ),
                2 => {
                    let (actual_map, expected_map) = &mut siblings[source];
                    let actual = actual_map.get_mut(&key);
                    let expected = expected_map.get_mut(&key);
                    assert_eq!(actual.is_some(), expected.is_some());
                    if let (Some(actual), Some(expected)) = (actual, expected) {
                        *actual = actual.wrapping_add(1);
                        *expected = expected.wrapping_add(1);
                    }
                }
                3 => {
                    let child = siblings[source].0.fork_persistent();
                    install_model_sibling(&mut siblings, source, child, random);
                }
                4 => {
                    let child = siblings[source].0.operational_clone();
                    install_model_sibling(&mut siblings, source, child, random);
                }
                _ => assert_eq!(siblings[source].0.get(&key), siblings[source].1.get(&key)),
            }
            for (actual, expected) in &siblings {
                assert_matches_model(actual, expected);
            }
        }
    }
}

fn install_model_sibling(
    siblings: &mut Vec<(PersistentOrdMap<u64, u64>, BTreeMap<u64, u64>)>,
    source: usize,
    child: PersistentOrdMap<u64, u64>,
    random: u64,
) {
    let model = siblings[source].1.clone();
    if siblings.len() < 6 {
        siblings.push((child, model));
    } else {
        let destination = (source + 1 + random as usize % (siblings.len() - 1)) % siblings.len();
        siblings[destination] = (child, model);
    }
}

fn assert_matches_model(actual: &PersistentOrdMap<u64, u64>, expected: &BTreeMap<u64, u64>) {
    assert_eq!(actual.len(), expected.len());
    assert_eq!(actual.first_key_value(), expected.first_key_value());
    assert_eq!(
        actual.iter().collect::<Vec<_>>(),
        expected.iter().collect::<Vec<_>>()
    );
    for key in [0, 1, 31, 63, 127] {
        assert_eq!(actual.get(&key), expected.get(&key));
    }
}
