use std::{collections::BTreeMap, num::NonZeroUsize, sync::OnceLock};

use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionMap,
    ExecutionResourceLease, ExecutionWorkCeiling, KeylessPartition, LeaseDenial, LeaseRequest,
    MapKernelContext, MapKernelFailure, MapOutcome, MapPartition, MapStop, MemoryLimitDenial,
    MemoryLimitLevel, SerialMemoryBudget, SerialRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

const PROCESS_BYTES: u64 = 1 << 30;

static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();

fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(2).unwrap(),
            charged_memory_bytes: PROCESS_BYTES,
        })
        .unwrap()
    })
}

fn policy(bytes: u64) -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), bytes, u64::MAX),
    )
}

fn request(bytes: u64) -> LeaseRequest {
    LeaseRequest {
        policy: policy(bytes),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

fn lease(bytes: u64) -> ExecutionResourceLease<'static> {
    authority().request_lease(request(bytes)).unwrap()
}

#[test]
fn the_authority_reports_its_configuration() {
    let config = authority().config();
    assert_eq!(config.max_workers.get(), 2);
    assert_eq!(config.charged_memory_bytes, PROCESS_BYTES);
}

#[test]
fn a_serial_budget_holds_the_policy_bytes_and_names_what_it_admits() {
    let budget = SerialMemoryBudget::from_policy(&policy(100));
    let mut first = budget.reserve(60).unwrap();
    assert_eq!(
        budget.reserve(41).unwrap_err(),
        MemoryLimitDenial {
            requested: 41,
            admitted: 40,
            level: MemoryLimitLevel::Policy { ancestor: 0 },
        }
    );
    let second = budget.reserve(40).unwrap();
    assert_eq!(
        first.resize(61).unwrap_err(),
        MemoryLimitDenial {
            requested: 61,
            admitted: 60,
            level: MemoryLimitLevel::Policy { ancestor: 0 },
        }
    );
    assert_eq!(first.bytes(), 60, "a refused resize keeps what was held");
    first.resize(10).unwrap();
    drop(second);
    assert_eq!(budget.reserve(90).unwrap().bytes(), 90);
}

#[test]
fn a_lease_reservation_is_bounded_by_its_lineage_and_released_on_drop() {
    let parent = lease(1_000);
    let child = parent.child(request(300)).unwrap();
    let held = parent.reserve_memory(800).unwrap();
    assert_eq!(
        child.reserve_memory(201).unwrap_err(),
        MemoryLimitDenial {
            requested: 201,
            admitted: 200,
            level: MemoryLimitLevel::Policy { ancestor: 1 },
        },
        "the parent's remaining room bounds the child"
    );
    let mut on_child = child.reserve_memory(200).unwrap();
    assert_eq!(
        on_child.resize(301).unwrap_err(),
        MemoryLimitDenial {
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
        MemoryLimitDenial {
            requested: 1,
            admitted: 0,
            level: MemoryLimitLevel::Policy { ancestor: 0 },
        }
    );
    drop(on_child);
    assert_eq!(child.reserve_memory(300).unwrap().bytes(), 300);
}

fn one_partition_map() -> ExecutionMap<u64, u64> {
    ExecutionMap::try_from_declared_partitions(
        vec![PartitionIdentity::new(1)],
        vec![MapPartition {
            identity: PartitionIdentity::new(1),
            value: 5,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 256,
            max_result_bytes: 0,
        }],
    )
    .unwrap()
}

fn serial_request(memory: SerialMemoryBudget) -> SerialRequest {
    SerialRequest::from_memory(memory, CancellationToken::new(), None)
}

/// `Some(true)` completed, `Some(false)` the map refused at its admission,
/// `None` the scope refused before the computation.
fn serial_run(bytes: u64) -> Option<bool> {
    serial_run_holding(bytes, None)
}

/// [`serial_run`], with `hold` bytes held on the budget before the run and
/// taken over by the map.
fn serial_run_holding(bytes: u64, hold: Option<u64>) -> Option<bool> {
    let map = one_partition_map();
    let budget = SerialMemoryBudget::from_policy(&policy(bytes));
    let held = hold.map(|hold| budget.reserve(hold).unwrap());
    let request = serial_request(budget);
    let ran = ExecutionWorkCeiling::new(u64::MAX).run_serial(&request, || {
        let kernel =
            |value: &u64, _: &mut MapKernelContext<'_, '_>| Ok::<_, MapKernelFailure<()>>(*value);
        match held {
            Some(held) => map.run_taking(None, held, kernel),
            None => map.run(None, kernel),
        }
    });
    match ran {
        Ok((MapOutcome::Complete { values, .. }, _)) => {
            assert_eq!(values, vec![5]);
            Some(true)
        }
        Ok((
            MapOutcome::Stopped {
                reason:
                    MapStop::Admission(LeaseDenial::MemoryExhausted(MemoryLimitDenial {
                        requested,
                        admitted,
                        level: MemoryLimitLevel::Policy { ancestor: 0 },
                    })),
                completed_prefix,
                ..
            },
            _,
        )) => {
            assert!(completed_prefix.is_empty(), "refused before the kernel ran");
            assert!(requested > admitted, "the budget names its own refusal");
            Some(false)
        }
        Ok(_) => panic!("unexpected serial map outcome"),
        Err(_) => None,
    }
}

#[test]
fn a_serial_run_with_a_policy_refuses_at_its_own_memory_boundary() {
    assert_eq!(serial_run(0), None, "the scope holds its framework bytes");
    let (mut refused, mut admitted) = (0_u64, 1 << 20);
    assert_eq!(serial_run(admitted), Some(true));
    while admitted - refused > 1 {
        let middle = refused + (admitted - refused) / 2;
        if serial_run(middle) == Some(true) {
            admitted = middle;
        } else {
            refused = middle;
        }
    }
    assert_eq!(
        serial_run(admitted - 1),
        Some(false),
        "one byte short refuses"
    );
    assert_eq!(serial_run(admitted), Some(true));

    let funded = ExecutionWorkCeiling::new(u64::MAX).run_serial(
        &serial_request(SerialMemoryBudget::from_policy(&policy(1 << 20))),
        || one_partition_map().run(None, |value, _| Ok::<_, MapKernelFailure<()>>(*value)),
    );
    assert!(matches!(funded, Ok((MapOutcome::Complete { .. }, _))));
}

#[test]
fn a_keyless_map_admits_what_the_declared_form_admits() {
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

/// The least budget `admits` admits, between 0 and 1 MiB.
fn least_admitted(admits: impl Fn(u64) -> bool) -> u64 {
    let (mut refused, mut admitted) = (0_u64, 1 << 20);
    assert!(admits(admitted));
    while admitted - refused > 1 {
        let middle = refused + (admitted - refused) / 2;
        if admits(middle) {
            admitted = middle;
        } else {
            refused = middle;
        }
    }
    admitted
}

/// The one-partition map on a child of a `budget`-byte lease while `hold`
/// bytes are held on the parent, as a request holds what it gathered: taken
/// over by the run, or kept beside it. `Err` is the run's memory refusal.
fn leased_run(budget: u64, hold: u64, take: bool) -> Result<(), MemoryLimitDenial> {
    let parent = lease(budget);
    let child = parent.child(request(budget)).unwrap();
    let held = parent.reserve_memory(hold).unwrap();
    let kernel =
        |value: &u64, _: &mut MapKernelContext<'_, '_>| Ok::<_, MapKernelFailure<()>>(*value);
    let outcome = if take {
        one_partition_map().run_taking(Some(&child), held, kernel)
    } else {
        let outcome = one_partition_map().run(Some(&child), kernel);
        drop(held);
        outcome
    };
    let result = match outcome {
        MapOutcome::Complete { values, .. } => {
            assert_eq!(values, vec![5]);
            Ok(())
        }
        MapOutcome::Stopped {
            reason: MapStop::Admission(LeaseDenial::MemoryExhausted(denial)),
            ..
        } => Err(denial),
        MapOutcome::Stopped { .. } => panic!("only memory refuses this run"),
    };
    assert_eq!(
        parent.reserve_memory(budget).map(|held| held.bytes()),
        Ok(budget),
        "nothing stays held once the run settles"
    );
    result
}

/// A run's admission takes the caller's hold on its inputs over in one
/// ledger step, from the parent onto the run's lease: it needs no room beside
/// the hold, and a refusal counts the hold as the run's own.
#[test]
fn a_leased_run_takes_over_the_hold_on_its_inputs() {
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
        Err(MemoryLimitDenial {
            requested: run,
            admitted: run - hold,
            level: MemoryLimitLevel::Policy { ancestor: 1 },
        }),
        "a hold kept beside the run leaves it short"
    );
    assert_eq!(
        leased_run(run - 1, hold, true),
        Err(MemoryLimitDenial {
            requested: run,
            admitted: run - 1,
            level: MemoryLimitLevel::Policy { ancestor: 0 },
        })
    );
}

/// The same on a serial budget: the scope's framework bytes and the hold fit
/// together, and the map then fits in the hold's place.
#[test]
fn a_serial_run_takes_over_the_hold_on_its_inputs() {
    let admitted = least_admitted(|budget| serial_run(budget) == Some(true));
    let hold = 1;
    assert_eq!(serial_run_holding(admitted, Some(hold)), Some(true));
    assert_eq!(serial_run_holding(admitted - 1, Some(hold)), Some(false));
}

/// A reduction takes over the caller's tree hold and leaves the completed
/// tree's bytes on it, so the tree is held from the run until the caller
/// releases it.
#[test]
fn a_reduced_tree_stays_held_on_the_callers_reservation() {
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
        Err(MemoryLimitDenial {
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
