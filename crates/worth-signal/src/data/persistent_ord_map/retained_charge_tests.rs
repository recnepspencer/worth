use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::data::retained_storage::{
    RetainedStorageMeasurement, RetainedStoragePreparation as Preparation,
    RetainedStoragePreparationDenial as Denial,
};

use super::{
    PersistentOrdMap, RetainedMapMutationDenial as MutationDenial,
    RetainedMapMutationOutcome as Outcome,
};

fn work() -> Preparation {
    Preparation::new(100_000)
}

fn assert_charge<K, V>(map: &PersistentOrdMap<K, V>)
where
    K: Clone + Ord + RetainedStorageMeasurement,
    V: Clone + RetainedStorageMeasurement,
{
    assert_eq!(
        map.prepared_retained_charge().unwrap(),
        map.retained_heap_charge(&mut work()).unwrap()
    );
}

fn accounted<R>(outcome: Result<Outcome<R>, MutationDenial>) -> R {
    match outcome.unwrap() {
        Outcome::Accounted { output, .. } => output,
        Outcome::Unaccounted { denial, .. } => panic!("unexpected accounting failure: {denial:?}"),
    }
}

#[test]
fn tracked_mutations_preserve_hidden_base_and_interval_payload_charges() {
    let mut base: PersistentOrdMap<String, String> = (0..9)
        .map(|n| {
            let mut key = String::with_capacity(1024 + n * 128);
            key.push_str(&format!("key-{n}"));
            let mut value = String::with_capacity(8192);
            value.push_str("original");
            (key, value)
        })
        .collect();
    base.prepare_retained_charge(&mut work()).unwrap();
    assert_charge(&base);
    accounted(
        base.edit_with_retained_charge(&"key-0".into(), &mut work(), |value| value.reserve(16384)),
    );
    assert_charge(&base);
    let mut selected = base.fork_persistent();
    assert_charge(&base);
    assert_charge(&selected);
    let retained = selected.clone();
    // Separate retirement intervals, then merge both sides. Readmission in
    // their middle splits them again, including freshly cloned endpoint keys.
    for n in [1, 3, 2, 5, 7, 6, 4, 0, 8] {
        assert!(
            accounted(selected.remove_with_retained_charge(&format!("key-{n}"), &mut work()))
                .is_some()
        );
        assert_charge(&selected);
        assert_charge(&retained);
    }
    assert!(selected.is_empty());
    drop(base);
    drop(retained);
    assert!(selected.prepared_retained_charge().unwrap().bytes() >= 9 * 8192);
    for n in [4, 0, 8, 2, 6, 1, 3, 5, 7] {
        accounted(selected.insert_with_retained_charge(
            format!("key-{n}"),
            "replacement".into(),
            &mut work(),
        ));
        assert_charge(&selected);
    }
    accounted(selected.insert_with_retained_charge(
        "outside".into(),
        String::with_capacity(32768),
        &mut work(),
    ));
    let fork = selected.fork_persistent();
    accounted(
        selected.edit_with_retained_charge(&"outside".into(), &mut work(), |value| {
            value.reserve(65536)
        }),
    );
    assert_charge(&selected);
    assert_charge(&fork);
    selected.clear();
    assert_charge(&selected);
    assert!(selected.prepared_retained_charge().unwrap().bytes() < 8192);
}

#[test]
fn raw_access_and_failed_measurement_never_expose_a_stale_fact() {
    let mut map: PersistentOrdMap<u32, String> = [(1, "one".into())].into_iter().collect();
    map.prepare_retained_charge(&mut work()).unwrap();
    let old = map.prepared_retained_charge().unwrap();
    assert!(matches!(
        map.edit_with_retained_charge(&1, &mut Preparation::new(0), |_| panic!("denied edit ran")),
        Err(MutationDenial::Accounting(Denial::WorkExhausted { .. }))
    ));
    assert_eq!(map.prepared_retained_charge().unwrap(), old);
    // Exclusive before-measurement visits map/key/value. No budget remains
    // after the edit: preserve its result and close accounting.
    match map
        .edit_with_retained_charge(&1, &mut Preparation::new(3), |value| {
            value.push_str(" changed");
            Box::new(42)
        })
        .unwrap()
    {
        Outcome::Unaccounted {
            output,
            denial: Denial::WorkExhausted { .. },
        } => assert_eq!(*output, 42),
        other => panic!("unexpected outcome: {other:?}"),
    }
    assert_eq!(map.get(&1).unwrap(), "one changed");
    assert_eq!(
        map.prepared_retained_charge(),
        Err(MutationDenial::PreparationRequired)
    );
    assert!(map
        .prepare_retained_charge(&mut Preparation::new(0))
        .is_err());
    map.prepare_retained_charge(&mut work()).unwrap();
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _ = map.edit_with_retained_charge(&1, &mut work(), |value| {
            value.reserve(32768);
            panic!("edit unwind");
        });
    }));
    assert!(result.is_err());
    assert_eq!(
        map.prepared_retained_charge(),
        Err(MutationDenial::PreparationRequired)
    );
    map.prepare_retained_charge(&mut work()).unwrap();
    map.entry(1).and_modify(|value| value.reserve(65536));
    assert_eq!(
        map.prepared_retained_charge(),
        Err(MutationDenial::PreparationRequired)
    );
    assert!(matches!(
        map.insert_with_retained_charge(2, "two".into(), &mut work()),
        Err(MutationDenial::PreparationRequired)
    ));
    assert!(!map.contains_key(&2));
    map.prepare_retained_charge(&mut work()).unwrap();
    assert_charge(&map);
    assert_eq!(
        map.operational_clone().prepared_retained_charge(),
        Err(MutationDenial::PreparationRequired)
    );
}

#[test]
fn mixed_fork_mutations_match_flat_values_and_full_representation_measurement() {
    let mut flat: BTreeMap<u32, Vec<String>> =
        (0..32).map(|n| (n, vec![format!("value-{n}")])).collect();
    let mut map: PersistentOrdMap<_, _> = flat.clone().into_iter().collect();
    map.prepare_retained_charge(&mut work()).unwrap();
    let mut pinned = Vec::new();
    for step in 0..384u32 {
        let key = (step * 17 + step / 5) % 40;
        match step % 4 {
            0 => {
                let value = vec![format!("replacement-{step}")];
                assert_eq!(
                    accounted(map.insert_with_retained_charge(key, value.clone(), &mut work())),
                    flat.insert(key, value)
                );
            }
            1 => assert_eq!(
                accounted(map.remove_with_retained_charge(&key, &mut work())),
                flat.remove(&key)
            ),
            2 if flat.contains_key(&key) => {
                accounted(map.edit_with_retained_charge(&key, &mut work(), |value| {
                    value.push("appended".into())
                }));
                flat.get_mut(&key).unwrap().push("appended".into());
            }
            _ => {}
        }
        if step % 48 == 0 {
            pinned.push((map.fork_persistent(), flat.clone()));
        }
        assert_eq!(
            map.iter().collect::<Vec<_>>(),
            flat.iter().collect::<Vec<_>>()
        );
        assert_charge(&map);
    }
    for (map, flat) in pinned {
        assert_eq!(
            map.iter().collect::<Vec<_>>(),
            flat.iter().collect::<Vec<_>>()
        );
        assert_charge(&map);
    }
}

#[test]
fn accounting_traversal_depends_on_selected_payload_not_untouched_population() {
    let mut visits = Vec::new();
    for population in [1, 8, 64] {
        let mut map: PersistentOrdMap<u32, String> =
            (0..population).map(|n| (n, "value".into())).collect();
        map.prepare_retained_charge(&mut work()).unwrap();
        let retained = map.fork_persistent();
        let mut edit_work = work();
        accounted(map.edit_with_retained_charge(&0, &mut edit_work, |value| value.push('!')));
        visits.push(edit_work.visits());
        let mut ready_work = Preparation::new(0);
        for _ in 0..64 {
            map.prepare_retained_charge(&mut ready_work).unwrap();
        }
        assert_eq!(ready_work.visits(), 0);
        assert_charge(&map);
        assert_charge(&retained);
    }
    assert!(visits[0] > 0);
    assert_eq!(visits, vec![visits[0]; 3]);
}

#[test]
fn empty_materialization_carries_constructor_charge_without_source_measurement() {
    let mut map = super::PersistentOrdMap::<u64, String>::new();
    let expected = map.prepared_retained_charge().unwrap();
    assert_eq!(map.clone().prepared_retained_charge().unwrap(), expected);
    map.insert(1, "retired payload".into());
    let _fork = map.fork_persistent();
    map.remove(&1);
    assert_eq!(
        map.operational_clone().prepared_retained_charge().unwrap(),
        expected
    );
}
