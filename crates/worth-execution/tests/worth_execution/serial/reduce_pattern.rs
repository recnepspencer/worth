#[allow(unused_imports)]
pub(crate) use crate::fast::reduce_pattern::*;

#[test]
fn native_reducer_uses_two_workers_and_matches_one_worker_oracle() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = test_serialization::guard(&TEST_LOCK);
    let input = map_with_count(64);
    let (serial_tree, serial_report, serial_metrics) = input
        .run_reduce(Some(&lease_with_workers(10_000, 1)), mapped, 0, sum, 8, 0)
        .unwrap();
    let active = AtomicUsize::new(0);
    let peak = AtomicUsize::new(0);
    let overlap = reducer_overlap::ReducerOverlap::new(2);
    let reduce = |left: &u64, right: &u64| {
        let now = active.fetch_add(1, Ordering::AcqRel) + 1;
        peak.fetch_max(now, Ordering::AcqRel);
        overlap.rendezvous();
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
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = test_serialization::guard(&TEST_LOCK);
    // For identities 1..=10, the canonical priority minimum is identity 10.
    // Its only child contains branches, which the frontier can schedule.
    let input = map_with_count(10);
    let (oracle, serial_report, serial_metrics) = input
        .run_reduce(Some(&lease_with_workers(10_000, 1)), mapped, 0, sum, 8, 0)
        .unwrap();
    let active = AtomicUsize::new(0);
    let peak = AtomicUsize::new(0);
    let overlap = reducer_overlap::ReducerOverlap::new(2);
    let reducer = |left: &u64, right: &u64| {
        let now = active.fetch_add(1, Ordering::AcqRel) + 1;
        peak.fetch_max(now, Ordering::AcqRel);
        overlap.rendezvous();
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

#[test]
fn work_ceiling_stops_inside_reduction_after_checked_map() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = test_serialization::guard(&TEST_LOCK);
    let (_, full_report, _) = map().run_reduce(None, mapped, 0, sum, 8, 0).unwrap();
    let ceiling = full_report.charged_work() - 1;
    let stopped = map().run_reduce(Some(&lease(ceiling)), mapped, 0, sum, 8, 0);
    match stopped {
        Err(ReduceInputDenial::ReductionStopped { failure, report }) => {
            assert_eq!(
                failure.reason,
                ReductionRunStop::Hook(worth_execution::MapKernelStop::WorkCeiling)
            );
            let accounting = failure
                .metrics
                .accounted_metrics()
                .expect("a leased reduction stop carries accounting");
            assert!(accounting.combine_calls > 0);
            assert_eq!(report.charged_work(), ceiling);
        }
        _ => panic!("expected reduction-stage exhaustion"),
    }
}

#[test]
fn complete_reduce_combines_map_work_with_tree_dependency_span() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = test_serialization::guard(&TEST_LOCK);
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
    let accounting = metrics
        .accounted_metrics()
        .expect("a leased reduction carries accounting");
    let serial_accounting = serial_metrics
        .accounted_metrics()
        .expect("a serial reduction carries accounting");
    assert_eq!(accounting.combine_calls, 6);
    assert_eq!(metrics, serial_metrics);
    assert_eq!(report.charged_work(), serial_report.charged_work());
    assert_eq!(report.charged_span(), 1 + accounting.charged_span);
    assert!(report.charged_span() < report.charged_work());
    assert_eq!(*serial_tree.result(), *tree.result());
    assert_eq!(
        serial_report.charged_span(),
        1 + serial_accounting.charged_span
    );
}

#[test]
fn map_failure_keeps_its_canonical_boundary_through_parent_scope() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = test_serialization::guard(&TEST_LOCK);
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
fn reduction_hook_error_bytes_obey_map_result_capacity() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = test_serialization::guard(&TEST_LOCK);
    let small = authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), Some(4096), Some(10)),
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
