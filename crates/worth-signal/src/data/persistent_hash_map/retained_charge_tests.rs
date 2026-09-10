use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::panic::{catch_unwind, AssertUnwindSafe};

use super::{
    PersistentHashMap, PersistentHashMapStorage, RetainedHashMutationDenial as MutationDenial,
    RetainedHashMutationOutcome as Outcome,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

fn work() -> Preparation {
    Preparation::new(100_000)
}

fn accounted<R>(outcome: Result<Outcome<R>, MutationDenial>) -> R {
    match outcome.unwrap() {
        Outcome::Accounted { output, .. } => output,
        Outcome::Unaccounted { denial, .. } => panic!("unexpected accounting failure: {denial:?}"),
    }
}

fn assert_charge<
    K: Clone + Eq + Hash + RetainedStorageMeasurement,
    V: Clone + RetainedStorageMeasurement,
>(
    map: &PersistentHashMap<K, V>,
) {
    assert_eq!(
        map.prepared_retained_charge().unwrap(),
        map.retained_heap_charge(&mut work()).unwrap()
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Key(u32, String);
impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        0u8.hash(state);
    }
}
impl RetainedStorageMeasurement for Key {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.1.retained_heap_charge(work)
    }
}
fn key(id: u32) -> Key {
    Key(id, "key".into())
}

#[test]
fn sparse_base_preparation_admits_retained_bucket_work_before_traversal() {
    let mut map: PersistentHashMap<u32, String> =
        (0..1024).map(|id| (id, format!("value-{id}"))).collect();
    for id in 0..1023 {
        map.remove(&id);
    }
    assert_eq!(map.len(), 1);
    assert!(map.base_capacity.unwrap() >= 1024);
    let mut sparse_work = Preparation::new(3);
    assert!(matches!(
        map.prepare_retained_charge(&mut sparse_work),
        Err(Denial::WorkExhausted { .. })
    ));
    assert_eq!(
        sparse_work.visits(),
        1,
        "bucket admission must precede payload traversal"
    );
    assert_eq!(
        map.prepared_retained_charge(),
        Err(MutationDenial::PreparationRequired)
    );
    map.prepare_retained_charge(&mut work()).unwrap();
    assert_eq!(map.get(&1023).unwrap(), "value-1023");
    assert_charge(&map);
    // Prepared, keyed mutation does not inherit the table-scan work bound.
    accounted(
        map.edit_with_retained_charge(&1023, &mut Preparation::new(6), |value| value.push('!')),
    );
    assert_charge(&map);
}

#[test]
fn accounted_collision_cow_growth_and_collapse_preserve_pinned_representations() {
    let mut empty = PersistentHashMap::new();
    let mut map = empty.fork_persistent();
    for id in 0..40 {
        let mut name = String::with_capacity(1024);
        name.push_str("key");
        accounted(map.insert_with_retained_charge(
            Key(id, name),
            String::with_capacity(8192),
            &mut work(),
        ));
    }
    let pinned = map.clone();
    // COW copies forty entries; insertion may grow that Vec to eighty, not
    // the original power-of-two capacity. Its charge must cover this path.
    accounted(map.insert_with_retained_charge(key(40), "new".into(), &mut work()));
    assert_charge(&map);
    assert_charge(&pinned);
    if let PersistentHashMapStorage::ForkShared {
        collision_extents, ..
    } = &map.storage
    {
        assert_eq!(collision_extents.as_ref().unwrap().counts(), (1, 1, 82));
    } else {
        unreachable!();
    }
    for id in 2..41 {
        accounted(map.remove_with_retained_charge(&key(id), &mut work()));
    }
    assert_charge(&map);
    let two = map.fork_persistent();
    accounted(map.edit_with_retained_charge(&key(0), &mut work(), |value| value.reserve(32768)));
    accounted(map.remove_with_retained_charge(&key(1), &mut work()));
    assert_charge(&map);
    assert_charge(&two);
    assert_eq!(pinned.len(), 40);
    assert_eq!(two.len(), 2);
    assert_eq!(map.len(), 1);
    assert!(pinned.prepared_retained_charge().unwrap().bytes() >= 40 * (8192 + 1024));
    map.clear();
    assert_eq!(map.prepared_retained_charge().unwrap(), Charge::ZERO);
}

#[test]
fn base_capacity_and_hidden_payload_survive_deletion_and_overlay_tombstones() {
    let mut map: PersistentHashMap<u32, Vec<String>> = (0..64)
        .map(|id| (id, vec![String::with_capacity(4096)]))
        .collect();
    map.prepare_retained_charge(&mut work()).unwrap();
    let initial_capacity = map.base_capacity;
    for id in 16..64 {
        accounted(map.remove_with_retained_charge(&id, &mut work()));
    }
    assert_eq!(map.base_capacity, initial_capacity);
    assert_charge(&map);
    let base = map.fork_persistent();
    for id in 0..16 {
        accounted(map.remove_with_retained_charge(&id, &mut work()));
    }
    assert!(map.is_empty());
    drop(base);
    assert!(map.prepared_retained_charge().unwrap().bytes() >= 16 * 4096);
    assert_charge(&map);
    for id in 0..16 {
        accounted(map.insert_with_retained_charge(id, vec!["readmitted".into()], &mut work()));
        assert_charge(&map);
    }
}

#[test]
fn raw_mutation_and_missing_extent_history_require_explicit_preparation_or_reconstruction() {
    let mut map: PersistentHashMap<u32, String> = [(1, "one".into())].into_iter().collect();
    map.prepare_retained_charge(&mut work()).unwrap();
    let before = map.prepared_retained_charge().unwrap();
    assert!(matches!(
        map.edit_with_retained_charge(&1, &mut Preparation::new(0), |_| panic!("denied edit ran")),
        Err(MutationDenial::Accounting(Denial::WorkExhausted { .. }))
    ));
    assert_eq!(map.prepared_retained_charge().unwrap(), before);
    match map
        .edit_with_retained_charge(&1, &mut Preparation::new(3), |value| {
            value.push_str(" changed");
            Box::new(42)
        })
        .unwrap()
    {
        Outcome::Unaccounted { output, .. } => assert_eq!(*output, 42),
        other => panic!("unexpected outcome: {other:?}"),
    }
    assert_eq!(map.get(&1).unwrap(), "one changed");
    assert_eq!(
        map.prepared_retained_charge(),
        Err(MutationDenial::PreparationRequired)
    );
    map.prepare_retained_charge(&mut work()).unwrap();
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = map.edit_with_retained_charge(&1, &mut work(), |value| {
            value.reserve(32768);
            panic!("unwind");
        });
    }))
    .is_err());
    assert_eq!(
        map.prepared_retained_charge(),
        Err(MutationDenial::PreparationRequired)
    );
    map.prepare_retained_charge(&mut work()).unwrap();
    map.entry(1).or_default().reserve(65536);
    assert_eq!(
        map.prepared_retained_charge(),
        Err(MutationDenial::PreparationRequired)
    );
    map.prepare_retained_charge(&mut work()).unwrap();
    let mut historical = map.fork_persistent();
    historical.get_mut(&1).unwrap().push('!');
    // Simulate unavailable allocation history after an interrupted overlay
    // mutation. Logical entries cannot reconstruct its retained Vec capacity.
    if let PersistentHashMapStorage::ForkShared {
        collision_extents, ..
    } = &mut historical.storage
    {
        *collision_extents = None;
    }
    assert_eq!(
        historical.prepare_retained_charge(&mut work()),
        Err(Denial::RetainedExtentHistoryUnavailable)
    );
    let mut reconstructed = historical.operational_clone();
    reconstructed.prepare_retained_charge(&mut work()).unwrap();
    assert_eq!(reconstructed, historical);
    assert_charge(&reconstructed);
}

#[test]
fn mixed_operations_match_independent_values_and_accounting_visits_ignore_population() {
    let mut flat: BTreeMap<u32, Vec<String>> = (0..32)
        .map(|id| (id, vec![format!("value-{id}")]))
        .collect();
    let mut map: PersistentHashMap<_, _> = flat.clone().into_iter().collect();
    map.prepare_retained_charge(&mut work()).unwrap();
    let mut pinned = Vec::new();
    for step in 0..256u32 {
        let id = (step * 17 + step / 5) % 40;
        if step % 2 == 0 {
            let value = vec![format!("replacement-{step}")];
            assert_eq!(
                accounted(map.insert_with_retained_charge(id, value.clone(), &mut work())),
                flat.insert(id, value)
            );
        } else {
            assert_eq!(
                accounted(map.remove_with_retained_charge(&id, &mut work())),
                flat.remove(&id)
            );
        }
        if step % 32 == 0 {
            pinned.push(map.fork_persistent());
        }
        assert_eq!(
            map.iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect::<BTreeMap<_, _>>(),
            flat
        );
        assert_charge(&map);
    }
    for map in pinned {
        assert_charge(&map);
    }
    let mut visits = Vec::new();
    for population in [1, 8, 64] {
        let mut map: PersistentHashMap<u32, String> =
            (0..population).map(|id| (id, "value".into())).collect();
        map.prepare_retained_charge(&mut work()).unwrap();
        let retained = map.fork_persistent();
        let mut edit_work = work();
        accounted(map.edit_with_retained_charge(&0, &mut edit_work, |value| value.push('!')));
        visits.push(edit_work.visits());
        for _ in 0..64 {
            map.prepare_retained_charge(&mut Preparation::new(0))
                .unwrap();
        }
        assert_charge(&map);
        assert_charge(&retained);
    }
    assert!(visits[0] > 0);
    assert_eq!(visits, vec![visits[0]; 3]);
}
