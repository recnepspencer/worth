#[allow(unused_imports)]
pub(crate) use crate::work_ceiling::isolated_context::retained_run;
#[allow(unused_imports)]
use crate::work_ceiling::ordinary_context::*;

pub(crate) use {std::num::NonZeroUsize, std::panic::catch_unwind, std::panic::AssertUnwindSafe};

pub(crate) use {
    worth_execution::CancellationSource, worth_execution::CancellationToken,
    worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig,
    worth_execution::ExecutionMap, worth_execution::ExecutionResourceLease,
    worth_execution::ExecutionWorkCeiling, worth_execution::LeaseRequest,
    worth_execution::MapKernelFailure, worth_execution::MapKernelStop,
    worth_execution::MapPartition, worth_execution::MapStop, worth_execution::ReduceInputDenial,
    worth_execution::ReductionRunStop, worth_execution::SerialRequest,
    worth_execution::WorkCeilingDenial,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionReport,
    worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity,
};

#[test]
fn a_declared_ceiling_decides_the_canonical_exhaustion_boundary_without_a_lease() {
    let (unbounded, unbounded_scope) = reduce(None, u64::MAX, 4);
    let (sum, report) = unbounded.expect("an unbounded ceiling completes");
    assert_eq!(sum, 10);
    let required = report.charged_work();
    assert!(required > 4 * PARTITION_WORK, "combines are charged too");
    assert_eq!(unbounded_scope.charged_work(), required);

    let (exact, exact_scope) = reduce(None, required, 4);
    assert_eq!(exact.expect("the exact ceiling completes").0, 10);
    assert_eq!(exact_scope.charged_work(), required);

    // Two partitions fit in eight units; the third is the boundary, every run.
    for _ in 0..3 {
        let (stopped, scope) = reduce(None, 2 * PARTITION_WORK + 2, 4);
        assert_eq!(exhausted_at(&stopped), Some(3));
        assert_eq!(scope.charged_work(), 2 * PARTITION_WORK);
    }
    let (first, _) = reduce(None, PARTITION_WORK - 1, 4);
    assert_eq!(exhausted_at(&first), Some(1));

    // Every partition fits and the reduction does not: the tree stops typed.
    let (reduction, _) = reduce(None, required - 1, 4);
    assert!(matches!(
        reduction,
        Err(worth_execution::ReduceInputDenial::ReductionStopped { ref failure, .. })
            if matches!(failure.reason, worth_execution::ReductionRunStop::Hook(_))
    ));
}

#[test]
fn a_panic_outside_every_pattern_is_typed_and_the_next_run_is_clean() {
    let panicked = catch_unwind(AssertUnwindSafe(|| {
        worth_execution::ExecutionWorkCeiling::new(Some(10))
            .run_serial(&funded_serial_request(), || -> u64 {
                panic!("outside a pattern")
            })
    }))
    .expect("the ceiling contains the panic");
    assert_eq!(
        panicked.unwrap_err(),
        worth_execution::WorkCeilingDenial::Panicked
    );
    let (after, _) = reduce(None, u64::MAX, 2);
    assert_eq!(after.expect("the next run completes").0, 3);
}

#[test]
fn a_serial_request_cancelled_mid_reduction_stops_at_a_combine() {
    let cancellation = worth_execution::CancellationSource::new();
    let request = funded_serial_request().with_cancellation(cancellation.token());
    let input = map(8);
    let combines = std::sync::atomic::AtomicU64::new(0);
    let (reduced, _) = worth_execution::ExecutionWorkCeiling::new(Some(u64::MAX))
        .run_serial(&request, || {
            input.run_reduce(
                None,
                |value, _| Ok::<_, MapKernelFailure<()>>(*value),
                0_u64,
                |left: &u64, right: &u64| {
                    combines.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    cancellation.cancel();
                    left + right
                },
                8,
                0,
            )
        })
        .expect("the computation ran");
    let Err(worth_execution::ReduceInputDenial::ReductionStopped { failure, .. }) = reduced else {
        panic!("cancellation stops the reduction");
    };
    assert_eq!(
        failure.reason,
        worth_execution::ReductionRunStop::Hook(worth_execution::MapKernelStop::Cancelled)
    );
    assert_eq!(
        combines.into_inner(),
        1,
        "the next combine boundary observes it"
    );
}

#[test]
fn a_serial_request_past_its_deadline_refuses_before_the_computation() {
    let request = funded_serial_request().with_deadline(Some(std::time::Instant::now()));
    let ran =
        worth_execution::ExecutionWorkCeiling::new(Some(u64::MAX)).run_serial(&request, || ());
    assert_eq!(
        ran.unwrap_err(),
        worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::DeadlineElapsed
        )
    );
}
