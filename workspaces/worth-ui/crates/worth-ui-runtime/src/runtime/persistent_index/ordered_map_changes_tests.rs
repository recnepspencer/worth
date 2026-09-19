use std::collections::{BTreeMap, BTreeSet};

use super::UiPersistentOrdMap;

#[test]
fn shared_versions_bound_local_comparison_work_at_multiple_scales() {
    for size in [64, 1_024, 16_384] {
        let mut current = UiPersistentOrdMap::default();
        for key in 0..size {
            current.insert(key, key);
        }
        let previous = current.clone();
        let (changed, work) = current.changed_keys_with_work(&previous);
        assert!(changed.is_empty());
        assert_eq!(work.cursor_steps(), 1);
        assert_eq!(work.shared_subtrees_skipped(), 1);

        current.insert(size / 2, -1);
        current.remove(&(size / 2 + 1));
        current.insert(size, size);
        let (changed, work) = current.changed_keys_with_work(&previous);
        assert_eq!(changed, [size / 2, size / 2 + 1, size]);
        assert!(work.shared_subtrees_skipped() > 0);
        assert!(work.cursor_steps() < 32 * (size as usize).ilog2() as usize);
        assert_eq!(previous.get(&(size / 2)), Some(&(size / 2)));
    }
}

#[test]
fn rotations_and_unrelated_roots_match_an_independent_map_oracle() {
    let mut current = UiPersistentOrdMap::default();
    let mut expected = BTreeMap::new();
    let mut seed = 17_u64;
    for _ in 0..80 {
        let previous = current.clone();
        let old_expected = expected.clone();
        for _ in 0..19 {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let key = (seed >> 32) % 257;
            if seed & 3 == 0 {
                current.remove(&key);
                expected.remove(&key);
            } else {
                current.insert(key, seed);
                expected.insert(key, seed);
            }
        }
        assert_changes(&current, &previous, &expected, &old_expected);
        assert_changes(&previous, &current, &old_expected, &expected);

        // Reverse insertion order deliberately destroys identity sharing and
        // changes tree shape while retaining independently held equal truth.
        let mut rebuilt = UiPersistentOrdMap::default();
        for (&key, &value) in expected.iter().rev() {
            rebuilt.insert(key, value);
        }
        assert!(current.changed_keys_with_work(&rebuilt).0.is_empty());
        assert_changes(&rebuilt, &previous, &expected, &old_expected);
    }
    let empty = UiPersistentOrdMap::default();
    assert_changes(&current, &empty, &expected, &BTreeMap::new());
    assert_changes(&empty, &current, &BTreeMap::new(), &expected);
}

fn assert_changes(
    current: &UiPersistentOrdMap<u64, u64>,
    previous: &UiPersistentOrdMap<u64, u64>,
    expected: &BTreeMap<u64, u64>,
    old_expected: &BTreeMap<u64, u64>,
) {
    let keys = expected
        .keys()
        .chain(old_expected.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    let changed = keys
        .into_iter()
        .filter(|key| expected.get(key) != old_expected.get(key))
        .collect::<Vec<_>>();
    assert_eq!(current.changed_keys_with_work(previous).0, changed);
}
