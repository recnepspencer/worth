use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
    time::Duration,
};

use worth_execution::{
    CancellationToken, CanonicalBits, ChargedBytes, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionMap, LeaseRequest, MapKernelFailure, MapOutcome, MapPartition, ReduceInputDenial,
    ReductionMetrics, ReductionRunFailure, ReductionRunStop,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
static TEST_LOCK: Mutex<()> = Mutex::new(());

fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(2).unwrap(),
            charged_memory_bytes: 1 << 20,
        })
        .unwrap()
    })
}

fn lease(work: u64) -> worth_execution::ExecutionResourceLease<'static> {
    lease_with_workers(work, 2)
}

fn lease_with_workers(
    work: u64,
    workers: usize,
) -> worth_execution::ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(workers).unwrap(), 1 << 20, work),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}

fn map() -> ExecutionMap<u64, u64> {
    map_with_count(3)
}

fn map_with_count(count: u64) -> ExecutionMap<u64, u64> {
    let ids: Vec<_> = (1..=count).map(PartitionIdentity::new).collect();
    let partitions = ids
        .iter()
        .copied()
        .enumerate()
        .map(|(index, identity)| MapPartition {
            identity,
            value: index as u64 + 1,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        })
        .collect();
    ExecutionMap::try_from_declared_partitions(ids, partitions).unwrap()
}

#[test]
fn native_reducer_uses_two_workers_and_matches_one_worker_oracle() {
    let _guard = TEST_LOCK.lock().unwrap();
    let input = map_with_count(64);
    let (serial_tree, serial_report, serial_metrics) = input
        .run_reduce(Some(&lease_with_workers(10_000, 1)), mapped, 0, sum, 8, 0)
        .unwrap();
    let active = AtomicUsize::new(0);
    let peak = AtomicUsize::new(0);
    let reduce = |left: &u64, right: &u64| {
        let now = active.fetch_add(1, Ordering::AcqRel) + 1;
        peak.fetch_max(now, Ordering::AcqRel);
        std::thread::sleep(Duration::from_millis(2));
        active.fetch_sub(1, Ordering::AcqRel);
        left + right
    };
    let (native_tree, native_report, native_metrics) = input
        .run_reduce(Some(&lease(10_000)), mapped, 0, reduce, 8, 0)
        .unwrap();
    assert_eq!(*native_tree.result(), *serial_tree.result());
    assert_eq!(native_metrics, serial_metrics);
    assert_eq!(native_report.charged_work(), serial_report.charged_work());
    assert_eq!(native_report.charged_span(), serial_report.charged_span());
    assert!(
        peak.load(Ordering::Acquire) > 1,
        "reducer calls must overlap"
    );
    assert!(native_report.physical().active_workers_high_watermark() > 1);
}

#[test]
fn edge_root_exposes_deeper_branches_without_changing_dependency_span() {
    let _guard = TEST_LOCK.lock().unwrap();
    // For identities 1..=10, the canonical priority minimum is identity 10.
    // Its only child contains branches, which the frontier can schedule.
    let input = map_with_count(10);
    let (oracle, serial_report, serial_metrics) = input
        .run_reduce(Some(&lease_with_workers(10_000, 1)), mapped, 0, sum, 8, 0)
        .unwrap();
    let active = AtomicUsize::new(0);
    let peak = AtomicUsize::new(0);
    let reducer = |left: &u64, right: &u64| {
        let now = active.fetch_add(1, Ordering::AcqRel) + 1;
        peak.fetch_max(now, Ordering::AcqRel);
        std::thread::sleep(Duration::from_millis(1));
        active.fetch_sub(1, Ordering::AcqRel);
        left + right
    };
    let (native, native_report, native_metrics) = input
        .run_reduce(Some(&lease(10_000)), mapped, 0, reducer, 8, 0)
        .unwrap();
    assert_eq!(*native.result(), *oracle.result());
    assert_eq!(native_metrics, serial_metrics);
    assert_eq!(native_report.charged_work(), serial_report.charged_work());
    assert_eq!(native_report.charged_span(), serial_report.charged_span());
    assert!(native_report.charged_span() < native_report.charged_work());
    assert!(peak.load(Ordering::Acquire) > 1);
}

fn mapped(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<u64, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(*value)
}

fn sum(left: &u64, right: &u64) -> u64 {
    left + right
}

#[test]
fn work_ceiling_stops_inside_reduction_after_checked_map() {
    let _guard = TEST_LOCK.lock().unwrap();
    let (_, full_report, _) = map().run_reduce(None, mapped, 0, sum, 8, 0).unwrap();
    let ceiling = full_report.charged_work() - 1;
    let stopped = map().run_reduce(Some(&lease(ceiling)), mapped, 0, sum, 8, 0);
    match stopped {
        Err(ReduceInputDenial::ReductionStopped { failure, report }) => {
            assert_eq!(
                failure.reason,
                ReductionRunStop::Hook(worth_execution::MapKernelStop::WorkCeiling)
            );
            assert!(failure.metrics.combine_calls > 0);
            assert_eq!(report.charged_work(), ceiling);
        }
        _ => panic!("expected reduction-stage exhaustion"),
    }
}

#[test]
fn complete_reduce_combines_map_work_with_tree_dependency_span() {
    let _guard = TEST_LOCK.lock().unwrap();
    let (serial_tree, serial_report, serial_metrics) =
        map().run_reduce(None, mapped, 0, sum, 8, 0).unwrap();
    let (tree, report, metrics) = map()
        .run_reduce(
            Some(&lease(serial_report.charged_work())),
            mapped,
            0,
            sum,
            8,
            0,
        )
        .unwrap();
    assert_eq!(*tree.result(), 6);
    assert_eq!(metrics.combine_calls, 6);
    assert_eq!(metrics, serial_metrics);
    assert_eq!(report.charged_work(), serial_report.charged_work());
    assert_eq!(report.charged_span(), 1 + metrics.charged_span);
    assert!(report.charged_span() < report.charged_work());
    assert_eq!(*serial_tree.result(), *tree.result());
    assert_eq!(
        serial_report.charged_span(),
        1 + serial_metrics.charged_span
    );
}

#[test]
fn map_failure_keeps_its_canonical_boundary_through_parent_scope() {
    let _guard = TEST_LOCK.lock().unwrap();
    let result = map().run_reduce(
        Some(&lease(9)),
        |value, context| {
            context.checkpoint(1)?;
            if *value == 2 {
                Err(MapKernelFailure::Domain("bad partition"))
            } else {
                Ok(*value)
            }
        },
        0,
        sum,
        8,
        0,
    );
    match result {
        Err(ReduceInputDenial::MapStopped {
            boundary,
            reason,
            report,
        }) => {
            assert_eq!(boundary, Some(PartitionIdentity::new(2)));
            assert!(matches!(
                reason,
                worth_execution::MapStop::Failure {
                    cause: MapKernelFailure::Domain("bad partition"),
                    ..
                }
            ));
            assert_eq!(report.charged_work(), 2);
        }
        _ => panic!("expected canonical map failure"),
    }
}

#[test]
fn no_lease_nested_reduction_charges_its_enclosing_map() {
    let _guard = TEST_LOCK.lock().unwrap();
    let (_, inner_report, _) = map().run_reduce(None, mapped, 0, sum, 8, 0).unwrap();
    let outer = map_with_count(1).run(None, |_, context| {
        context.checkpoint(1)?;
        let (_, report, _) = map()
            .run_reduce(None, mapped, 0, sum, 8, 0)
            .map_err(|_| MapKernelFailure::Domain(()))?;
        Ok::<_, MapKernelFailure<()>>(report.charged_work())
    });
    match outer {
        MapOutcome::Complete { values, report } => {
            assert_eq!(values, vec![inner_report.charged_work()]);
            assert_eq!(report.charged_work(), inner_report.charged_work() + 1);
        }
        MapOutcome::Stopped { .. } => panic!("nested serial reduction must complete"),
    }
}

struct PanickingClone(u64);

impl Clone for PanickingClone {
    fn clone(&self) -> Self {
        panic!("reduction clone panic")
    }
}

impl ChargedBytes for PanickingClone {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl CanonicalBits for PanickingClone {
    fn canonical_len(&self) -> Option<usize> {
        Some(8)
    }

    fn visit_canonical_bits(&self, visit: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visit(&self.0.to_le_bytes())
    }
}

#[test]
fn no_lease_reduction_contains_clone_panic() {
    let _guard = TEST_LOCK.lock().unwrap();
    let caught = std::panic::catch_unwind(|| {
        map().run_reduce(
            None,
            |value, _| Ok::<_, MapKernelFailure<()>>(PanickingClone(*value)),
            PanickingClone(0),
            |_, _| PanickingClone(0),
            8,
            0,
        )
    });
    assert!(matches!(
        caught,
        Ok(Err(ReduceInputDenial::ReductionStopped {
            failure: ReductionRunFailure {
                reason: ReductionRunStop::Panic,
                ..
            },
            ..
        }))
    ));
}

#[test]
fn reduction_hook_error_bytes_obey_map_result_capacity() {
    let _guard = TEST_LOCK.lock().unwrap();
    let small = authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), 4096, 10),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let outcome = map_with_count(1).run(Some(&small), |_, _| {
        Err::<u64, _>(MapKernelFailure::Domain(ReductionRunFailure {
            reason: ReductionRunStop::Hook("x".repeat(1 << 20)),
            metrics: ReductionMetrics::default(),
        }))
    });
    assert!(matches!(
        outcome,
        MapOutcome::Stopped {
            reason: worth_execution::MapStop::Failure {
                cause: MapKernelFailure::ResultCapacityExceeded,
                ..
            },
            ..
        }
    ));
}
