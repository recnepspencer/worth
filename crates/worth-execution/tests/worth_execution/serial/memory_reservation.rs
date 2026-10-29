#[allow(unused_imports)]
pub(crate) use crate::fast::memory_reservation::*;

#[path = "memory_reservation/payload_custody.rs"]
pub(crate) mod payload_custody;

#[path = "memory_reservation/process_limit.rs"]
pub(crate) mod process_limit;

#[test]
fn a_lease_reservation_is_bounded_by_its_lineage_and_released_on_drop() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = test_serialization::guard(&SERIAL);
    let parent = lease(1_000);
    let child = parent.child(request(300)).unwrap();
    let held = parent.reserve_memory(800).unwrap();
    assert_eq!(
        child.reserve_memory(201).unwrap_err(),
        MemoryLimitDenial::Capacity {
            requested: 201,
            admitted: 200,
            level: MemoryLimitLevel::Policy { ancestor: 1 },
        },
        "the parent's remaining room bounds the child"
    );
    let mut on_child = child.reserve_memory(200).unwrap();
    assert_eq!(
        on_child.resize(301).unwrap_err(),
        MemoryLimitDenial::Capacity {
            requested: 301,
            admitted: 300,
            level: MemoryLimitLevel::Policy { ancestor: 0 },
        },
        "the child's own limit bounds it too"
    );
    drop(held);
    on_child.resize(300).unwrap();
    assert_eq!(
        child.reserve_memory(1).unwrap_err(),
        MemoryLimitDenial::Capacity {
            requested: 1,
            admitted: 0,
            level: MemoryLimitLevel::Policy { ancestor: 0 },
        }
    );
    drop(on_child);
    assert_eq!(child.reserve_memory(300).unwrap().bytes(), 300);
}

#[test]
fn a_keyless_map_admits_what_the_declared_form_admits() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = test_serialization::guard(&SERIAL);
    let identities = [3, 1, 2].map(PartitionIdentity::new);
    let keyless: BTreeMap<_, _> = identities
        .iter()
        .map(|identity| {
            (
                *identity,
                KeylessPartition {
                    value: identity.value(),
                    kernel_scratch_bytes: 16,
                    max_result_bytes: 8,
                },
            )
        })
        .collect();
    let keyless = ExecutionMap::<u64, u64>::from_keyless_partitions(keyless).unwrap();
    let mut ordered = identities;
    ordered.sort();
    let declared = ExecutionMap::<u64, u64>::try_from_declared_partitions(
        ordered.to_vec(),
        ordered
            .iter()
            .map(|identity| MapPartition {
                identity: *identity,
                value: identity.value(),
                read_keys: Vec::new(),
                write_keys: Vec::new(),
                kernel_scratch_bytes: 16,
                max_result_bytes: 8,
            })
            .collect(),
    )
    .unwrap();
    assert_eq!(keyless.identities(), declared.identities());
    let lease = lease(1 << 20);
    let run = |map: &ExecutionMap<u64, u64>| match map.run(Some(&lease), |value, _| {
        Ok::<_, MapKernelFailure<()>>(*value)
    }) {
        MapOutcome::Complete { values, report } => (values, report),
        MapOutcome::Stopped { .. } => panic!("the map completes"),
    };
    assert_eq!(run(&keyless), run(&declared));
    assert_eq!(run(&keyless).0, vec![1, 2, 3]);
}

/// A run's admission takes the caller's hold on its inputs over in one
/// ledger step, from the parent onto the run's lease: it needs no room beside
/// the hold, and a refusal counts the hold as the run's own.
#[test]
fn a_leased_run_takes_over_the_hold_on_its_inputs() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = test_serialization::guard(&SERIAL);
    let run = least_admitted(|budget| leased_run(budget, 0, false).is_ok());
    let hold = run / 2;
    assert!(hold > 0);
    assert_eq!(
        leased_run(run, hold, true),
        Ok(()),
        "the hold becomes the run's"
    );
    assert_eq!(
        leased_run(run, hold, false),
        Err(MemoryLimitDenial::Capacity {
            requested: run,
            admitted: run - hold,
            level: MemoryLimitLevel::Policy { ancestor: 1 },
        }),
        "a hold kept beside the run leaves it short"
    );
    assert_eq!(
        leased_run(run - 1, hold, true),
        Err(MemoryLimitDenial::Capacity {
            requested: run,
            admitted: run - 1,
            level: MemoryLimitLevel::Policy { ancestor: 0 },
        })
    );
}

/// A reduction takes over the caller's tree hold and leaves the completed
/// tree's bytes on it, so the tree is held from the run until the caller
/// releases it.
#[test]
fn a_reduced_tree_stays_held_on_the_callers_reservation() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = test_serialization::guard(&SERIAL);
    let budget = 1 << 20;
    let parent = lease(budget);
    let child = parent.child(request(budget)).unwrap();
    let mut tree = parent.reserve_memory(0).unwrap();
    let kernel =
        |value: &u64, _: &mut MapKernelContext<'_, '_>| Ok::<_, MapKernelFailure<()>>(*value);
    let sum = |left: &u64, right: &u64| left + right;
    let reduced = one_partition_map().run_reduce_holding(
        Some(&child),
        None,
        Some(&mut tree),
        kernel,
        0,
        sum,
        8,
        0,
    );
    let (reduced, _, _) = reduced.unwrap_or_else(|_| panic!("the reduction completes"));
    assert_eq!(*reduced.result(), 5);
    let kept = tree.bytes();
    assert!(kept > 0, "the tree's bytes stay held");
    assert_eq!(
        parent.reserve_memory(budget).map(|held| held.bytes()),
        Err(MemoryLimitDenial::Capacity {
            requested: budget,
            admitted: budget - kept,
            level: MemoryLimitLevel::Policy { ancestor: 0 },
        }),
        "only the tree stays held once the run settles"
    );
    drop(tree);
    drop(reduced);
    assert_eq!(parent.reserve_memory(budget).unwrap().bytes(), budget);
}
