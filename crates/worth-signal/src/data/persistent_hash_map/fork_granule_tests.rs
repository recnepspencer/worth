use std::collections::HashMap;
use std::env;
use std::hash::{Hash, Hasher};
use std::hint::black_box;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use stats_alloc::{Region, INSTRUMENTED_SYSTEM};

use super::PersistentHashMap;

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

#[derive(Debug, Eq, PartialEq)]
struct CountedKeyItem(u64);

impl Clone for CountedKeyItem {
    fn clone(&self) -> Self {
        KEY_ITEM_CLONES.fetch_add(1, Ordering::Relaxed);
        Self(self.0)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct CountedKey {
    id: u64,
    payload: Vec<CountedKeyItem>,
}

impl Clone for CountedKey {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            payload: self.payload.clone(),
        }
    }
}

impl Hash for CountedKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        0_u8.hash(state);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StableKey(u64);

impl Hash for StableKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        0_u8.hash(state);
    }
}

#[test]
fn repeated_fork_unrelated_insert_copies_no_inherited_key_or_value_payload() {
    const CHILD: &str = "WORTH_SIGNAL_HASH_MAP_GRANULE_CHILD";
    const TEST: &str = "data::persistent_hash_map::fork_granule_tests::repeated_fork_unrelated_insert_copies_no_inherited_key_or_value_payload";
    if env::var_os(CHILD).is_none() {
        let output = Command::new(env::current_exe().expect("test executable resolves"))
            .arg("--exact")
            .arg(TEST)
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(CHILD, "1")
            .output()
            .expect("isolated hash-map granule probe starts");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        print!("{stdout}");
        eprint!("{stderr}");
        assert!(
            output.status.success()
                && stdout.contains(TEST)
                && stdout.contains("1 passed; 0 failed"),
            "hash-map granule probe did not run exactly once: stdout={stdout:?} stderr={stderr:?}"
        );
        return;
    }

    let mut allocations = Vec::new();
    for payload_size in [64_usize, 4_096, 65_536] {
        let value_allocation = prove_unrelated_value_insert(payload_size);
        let key_allocation = prove_unrelated_key_insert(payload_size);
        println!(
            "hash_map_payload={payload_size} value_calls={} value_bytes={} key_calls={} key_bytes={}",
            value_allocation.0, value_allocation.1, key_allocation.0, key_allocation.1
        );
        allocations.push((payload_size, value_allocation, key_allocation));
    }
    assert_bounded_handle_detachment(&allocations);
}

#[test]
fn get_mut_copies_only_the_selected_shared_value() {
    let mut original = PersistentHashMap::<u64, Vec<CountedValue>>::new();
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
    let mut original = PersistentHashMap::<StableKey, Vec<CountedValue>>::new();
    let mut source = original.fork_persistent();
    source.insert(
        StableKey(1),
        (0..payload_size as u64).map(CountedValue).collect(),
    );
    VALUE_ITEM_CLONES.store(0, Ordering::Relaxed);
    let mut sibling = source.fork_persistent();
    assert_eq!(VALUE_ITEM_CLONES.load(Ordering::Relaxed), 0);
    assert!(source.ptr_eq(&sibling));

    let region = Region::new(&INSTRUMENTED_SYSTEM);
    assert_eq!(
        sibling.insert(StableKey(2), vec![CountedValue(u64::MAX)]),
        None
    );
    let allocation = region.change();
    assert_eq!(VALUE_ITEM_CLONES.load(Ordering::Relaxed), 0);
    assert!(original.is_empty());
    assert_eq!(source.get(&StableKey(1)).map(Vec::len), Some(payload_size));
    assert_eq!(source.get(&StableKey(2)), None);
    assert_eq!(sibling.get(&StableKey(1)).map(Vec::len), Some(payload_size));
    assert_eq!(sibling.get(&StableKey(2)).map(Vec::len), Some(1));

    (allocation.allocations, allocation.bytes_allocated)
}

fn prove_unrelated_key_insert(payload_size: usize) -> (usize, usize) {
    let key = |id| CountedKey {
        id,
        payload: (0..payload_size).map(|_| CountedKeyItem(0)).collect(),
    };
    let mut original = PersistentHashMap::<CountedKey, u64>::new();
    let mut source = original.fork_persistent();
    source.insert(key(1), 7);
    KEY_ITEM_CLONES.store(0, Ordering::Relaxed);
    let mut sibling = source.fork_persistent();
    assert_eq!(KEY_ITEM_CLONES.load(Ordering::Relaxed), 0);
    assert!(source.ptr_eq(&sibling));

    let region = Region::new(&INSTRUMENTED_SYSTEM);
    let unrelated_key = || CountedKey {
        id: 2,
        payload: vec![CountedKeyItem(1)],
    };
    assert_eq!(sibling.insert(unrelated_key(), 8), None);
    let allocation = region.change();
    assert_eq!(KEY_ITEM_CLONES.load(Ordering::Relaxed), 0);
    assert!(original.is_empty());
    assert_eq!(source.get(&key(1)), Some(&7));
    assert_eq!(source.get(&unrelated_key()), None);
    assert_eq!(sibling.get(&key(1)), Some(&7));
    assert_eq!(sibling.get(&unrelated_key()), Some(&8));

    (allocation.allocations, allocation.bytes_allocated)
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
fn fork_overlay_preserves_hash_map_contracts_against_std_model() {
    let mut map: PersistentHashMap<u64, Vec<u64>> =
        [(1, vec![10]), (3, vec![30])].into_iter().collect();
    let mut model: HashMap<u64, Vec<u64>> = [(1, vec![10]), (3, vec![30])].into_iter().collect();
    let source = map.fork_persistent();

    map.entry(2).or_default().push(20);
    model.entry(2).or_default().push(20);
    map.get_mut(&1).expect("inherited value exists").push(11);
    model.get_mut(&1).unwrap().push(11);
    assert_eq!(map.insert(3, vec![300]), model.insert(3, vec![300]));
    assert_eq!(map.remove(&1), model.remove(&1));
    assert_eq!(map.insert(1, vec![100]), model.insert(1, vec![100]));
    for (key, expected) in &model {
        assert_eq!(map.get(key), Some(expected));
    }
    assert_eq!(map.len(), model.len());

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
    let decoded: PersistentHashMap<u64, Vec<u64>> = serde_json::from_str(&encoded).unwrap();
    assert_eq!(map, decoded);
    assert_eq!(source.get(&1), Some(&vec![10]));
    assert_eq!(source.get(&2), None);

    map.clear();
    model.clear();
    assert_eq!(map.is_empty(), model.is_empty());
    assert_eq!(map.iter().next(), None);
}
