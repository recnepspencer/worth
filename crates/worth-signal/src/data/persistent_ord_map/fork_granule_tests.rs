use std::collections::BTreeMap;
use std::env;
use std::hint::black_box;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use stats_alloc::{Region, INSTRUMENTED_SYSTEM};

use super::PersistentOrdMap;

static VALUE_ITEM_CLONES: AtomicUsize = AtomicUsize::new(0);
static KEY_ITEM_CLONES: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Eq, PartialEq)]
struct CountedValue(u64);

impl Clone for CountedValue {
    fn clone(&self) -> Self {
        VALUE_ITEM_CLONES.fetch_add(1, Ordering::Relaxed);
        Self(self.0)
    }
}

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct CountedKeyItem(u64);

impl Clone for CountedKeyItem {
    fn clone(&self) -> Self {
        KEY_ITEM_CLONES.fetch_add(1, Ordering::Relaxed);
        Self(self.0)
    }
}

#[test]
fn repeated_fork_unrelated_insert_copies_no_inherited_key_or_value_payload() {
    const CHILD: &str = "WORTH_SIGNAL_ORD_MAP_GRANULE_CHILD";
    const TEST: &str = "data::persistent_ord_map::fork_granule_tests::repeated_fork_unrelated_insert_copies_no_inherited_key_or_value_payload";
    if env::var_os(CHILD).is_none() {
        let output = Command::new(env::current_exe().expect("test executable resolves"))
            .arg("--exact")
            .arg(TEST)
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(CHILD, "1")
            .output()
            .expect("isolated ordered-map granule probe starts");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        print!("{stdout}");
        eprint!("{stderr}");
        assert!(
            output.status.success()
                && stdout.contains(TEST)
                && stdout.contains("1 passed; 0 failed"),
            "ordered-map granule probe did not run exactly once: stdout={stdout:?} stderr={stderr:?}"
        );
        return;
    }

    let mut allocations = Vec::new();
    for payload_size in [64_usize, 4_096, 65_536] {
        let value_allocation = prove_unrelated_value_insert(payload_size);
        let key_allocation = prove_unrelated_key_insert(payload_size);
        println!(
            "ordered_map_payload={payload_size} value_calls={} value_bytes={} key_calls={} key_bytes={}",
            value_allocation.0, value_allocation.1, key_allocation.0, key_allocation.1
        );
        allocations.push((payload_size, value_allocation, key_allocation));
    }
    prove_borrowed_retirement_lookups_do_not_allocate();
    assert_bounded_handle_detachment(&allocations);
}

#[test]
fn get_mut_copies_only_the_selected_shared_value() {
    let mut original = PersistentOrdMap::<u64, Vec<CountedValue>>::new();
    let mut source = original.fork_persistent();
    source.insert(1, (0..64).map(CountedValue).collect());
    source.insert(2, (0..4_096).map(CountedValue).collect());
    let mut sibling = source.fork_persistent();

    VALUE_ITEM_CLONES.store(0, Ordering::Relaxed);
    sibling.get_mut(&1).unwrap().push(CountedValue(64));
    assert_eq!(VALUE_ITEM_CLONES.load(Ordering::Relaxed), 64);
    assert_eq!(source.get(&1).map(Vec::len), Some(64));
    assert_eq!(source.get(&2).map(Vec::len), Some(4_096));
    assert_eq!(sibling.get(&1).map(Vec::len), Some(65));
    assert_eq!(sibling.get(&2).map(Vec::len), Some(4_096));
}

fn prove_unrelated_value_insert(payload_size: usize) -> (usize, usize) {
    let mut original = PersistentOrdMap::<u64, Vec<CountedValue>>::new();
    let mut source = original.fork_persistent();
    source.insert(1, (0..payload_size as u64).map(CountedValue).collect());
    VALUE_ITEM_CLONES.store(0, Ordering::Relaxed);
    let mut sibling = source.fork_persistent();
    assert_eq!(VALUE_ITEM_CLONES.load(Ordering::Relaxed), 0);
    assert!(source.ptr_eq(&sibling));

    let region = Region::new(&INSTRUMENTED_SYSTEM);
    assert_eq!(sibling.insert(2, vec![CountedValue(u64::MAX)]), None);
    let allocation = region.change();
    assert_eq!(VALUE_ITEM_CLONES.load(Ordering::Relaxed), 0);
    assert!(original.is_empty());
    assert_eq!(source.get(&1).map(Vec::len), Some(payload_size));
    assert_eq!(source.get(&2), None);
    assert_eq!(sibling.get(&1).map(Vec::len), Some(payload_size));
    assert_eq!(sibling.get(&2).map(Vec::len), Some(1));

    (allocation.allocations, allocation.bytes_allocated)
}

fn prove_unrelated_key_insert(payload_size: usize) -> (usize, usize) {
    let inherited_key = || {
        (0..payload_size)
            .map(|_| CountedKeyItem(0))
            .collect::<Vec<_>>()
    };
    let mut original = PersistentOrdMap::<Vec<CountedKeyItem>, u64>::new();
    let mut source = original.fork_persistent();
    source.insert(inherited_key(), 7);
    KEY_ITEM_CLONES.store(0, Ordering::Relaxed);
    let mut sibling = source.fork_persistent();
    assert_eq!(KEY_ITEM_CLONES.load(Ordering::Relaxed), 0);
    assert!(source.ptr_eq(&sibling));

    let region = Region::new(&INSTRUMENTED_SYSTEM);
    assert_eq!(sibling.insert(vec![CountedKeyItem(1)], 8), None);
    let allocation = region.change();
    assert_eq!(KEY_ITEM_CLONES.load(Ordering::Relaxed), 0);
    assert!(original.is_empty());
    assert_eq!(source.get(&inherited_key()), Some(&7));
    assert_eq!(source.get(&vec![CountedKeyItem(1)]), None);
    assert_eq!(sibling.get(&inherited_key()), Some(&7));
    assert_eq!(sibling.get(&vec![CountedKeyItem(1)]), Some(&8));

    (allocation.allocations, allocation.bytes_allocated)
}

fn prove_borrowed_retirement_lookups_do_not_allocate() {
    let mut original: PersistentOrdMap<String, u64> = [
        ("alpha".to_owned(), 1),
        ("beta".to_owned(), 2),
        ("gamma".to_owned(), 3),
    ]
    .into_iter()
    .collect();
    let mut sibling = original.fork_persistent();
    assert_eq!(sibling.remove(&"beta".to_owned()), Some(2));

    let region = Region::new(&INSTRUMENTED_SYSTEM);
    assert_eq!(black_box(sibling.get("alpha")), Some(&1));
    assert_eq!(black_box(sibling.get("beta")), None);
    assert_eq!(black_box(sibling.get("gamma")), Some(&3));
    assert_eq!(black_box(sibling.get("omega")), None);
    let allocation = region.change();
    println!(
        "ordered_map_borrowed_retirement_lookup_calls={} bytes={}",
        allocation.allocations, allocation.bytes_allocated
    );
    assert_eq!(allocation.allocations, 0);
    assert_eq!(allocation.bytes_allocated, 0);
}

fn assert_bounded_handle_detachment(samples: &[(usize, (usize, usize), (usize, usize))]) {
    for (kind, observations) in [
        (
            "value",
            samples.iter().map(|sample| sample.1).collect::<Vec<_>>(),
        ),
        (
            "key",
            samples.iter().map(|sample| sample.2).collect::<Vec<_>>(),
        ),
    ] {
        let minimum_calls = observations.iter().map(|sample| sample.0).min().unwrap();
        let minimum_bytes = observations.iter().map(|sample| sample.1).min().unwrap();
        for (calls, bytes) in observations {
            assert!(
                calls <= 16,
                "{kind} detachment exceeded 16 allocations: {calls}"
            );
            assert!(
                bytes <= 8 * 1_024,
                "{kind} detachment exceeded 8 KiB: {bytes}"
            );
            assert!(
                calls <= minimum_calls + 2,
                "{kind} allocation calls gained payload slope"
            );
            assert!(
                bytes <= minimum_bytes + 512,
                "{kind} allocation bytes gained payload slope"
            );
        }
    }
}

#[test]
fn fork_overlay_preserves_ordered_map_contracts_against_btree_model() {
    let mut map: PersistentOrdMap<u64, Vec<u64>> =
        [(1, vec![10]), (3, vec![30])].into_iter().collect();
    let mut model: BTreeMap<u64, Vec<u64>> = [(1, vec![10]), (3, vec![30])].into_iter().collect();
    let source = map.fork_persistent();

    map.entry(2).or_insert(vec![20]).push(21);
    model.entry(2).or_insert(vec![20]).push(21);
    map.entry(3).and_modify(|value| value.push(31));
    model.entry(3).and_modify(|value| value.push(31));
    map.entry(4).or_default().push(40);
    model.entry(4).or_default().push(40);
    map.get_mut(&1).expect("inherited value exists").push(11);
    model.get_mut(&1).unwrap().push(11);
    assert_eq!(map.insert(3, vec![300]), model.insert(3, vec![300]));
    assert_eq!(map.remove(&1), model.remove(&1));
    assert_eq!(map.insert(1, vec![100]), model.insert(1, vec![100]));
    assert_eq!(map.first_key_value(), model.first_key_value());
    assert_eq!(
        map.keys().copied().collect::<Vec<_>>(),
        model.keys().copied().collect::<Vec<_>>()
    );
    assert_eq!(
        map.values().cloned().collect::<Vec<_>>(),
        model.values().cloned().collect::<Vec<_>>()
    );

    let mut iter = map.iter();
    assert_eq!(iter.size_hint(), (model.len(), Some(model.len())));
    black_box(iter.next());
    assert_eq!(iter.size_hint(), (model.len() - 1, Some(model.len() - 1)));
    while iter.next().is_some() {}
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.next(), None);

    let shared = map.clone();
    assert!(map.ptr_eq(&shared));
    assert_eq!(map, shared);
    let reconstructed = map.operational_clone();
    assert!(!map.ptr_eq(&reconstructed));
    assert_eq!(map, reconstructed);
    let encoded = serde_json::to_string(&map).unwrap();
    let decoded: PersistentOrdMap<u64, Vec<u64>> = serde_json::from_str(&encoded).unwrap();
    assert_eq!(map, decoded);
    assert_eq!(source.get(&1), Some(&vec![10]));
    assert_eq!(source.get(&2), None);

    map.clear();
    model.clear();
    assert_eq!(map.is_empty(), model.is_empty());
    assert_eq!(map.iter().next(), None);
}

#[test]
fn borrowed_string_lookup_reaches_changed_and_retired_overlay_keys() {
    let mut map: PersistentOrdMap<String, u64> = [("alpha".to_owned(), 1), ("gamma".to_owned(), 3)]
        .into_iter()
        .collect();
    let source = map.fork_persistent();
    assert_eq!(map.insert("beta".to_owned(), 2), None);
    assert_eq!(map.get("beta"), Some(&2));
    assert_eq!(map.remove(&"alpha".to_owned()), Some(1));
    assert_eq!(map.get("alpha"), None);
    assert_eq!(map.insert("alpha".to_owned(), 10), None);
    assert_eq!(map.get("alpha"), Some(&10));
    assert_eq!(source.get("alpha"), Some(&1));
    assert_eq!(source.get("beta"), None);
}

#[test]
fn retired_interval_boundary_transitions_match_btree_truth() {
    prove_retirement_sequence(&[0, 1], &[]);
    prove_retirement_sequence(&[2, 1], &[]);
    prove_retirement_sequence(&[1, 3, 2], &[]);
    prove_retirement_sequence(&[1, 2, 3], &[2]);
}

fn prove_retirement_sequence(removals: &[u64], readmissions: &[u64]) {
    let mut map: PersistentOrdMap<u64, u64> = (0..=6).map(|key| (key, key)).collect();
    let mut model: BTreeMap<u64, u64> = (0..=6).map(|key| (key, key)).collect();
    let source = map.fork_persistent();

    for key in removals {
        assert_eq!(map.remove(key), model.remove(key));
        assert_eq!(
            map.iter()
                .map(|(key, value)| (*key, *value))
                .collect::<Vec<_>>(),
            model
                .iter()
                .map(|(key, value)| (*key, *value))
                .collect::<Vec<_>>()
        );
    }
    for key in readmissions {
        assert_eq!(map.insert(*key, key * 10), model.insert(*key, key * 10));
        assert_eq!(
            map.iter()
                .map(|(key, value)| (*key, *value))
                .collect::<Vec<_>>(),
            model
                .iter()
                .map(|(key, value)| (*key, *value))
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(source.len(), 7);
    assert_eq!(
        source.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
        (0..=6).collect::<Vec<_>>()
    );
}
