use std::{
    num::NonZeroUsize,
    panic::{catch_unwind, AssertUnwindSafe},
};

use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionMap, ExecutionResourceLease, ExecutionWorkCeiling, LeaseRequest, MapKernelFailure,
    MapKernelStop, MapPartition, MapStop, ReduceInputDenial, ReductionRunStop, SerialRequest,
    WorkCeilingDenial,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionReport,
    ExecutionRequestPolicy, PartitionIdentity,
};

const PARTITION_WORK: u64 = 3;

fn map(count: u64) -> ExecutionMap<u64, u64> {
    let ids: Vec<_> = (1..=count).map(PartitionIdentity::new).collect();
    let partitions = ids
        .iter()
        .map(|identity| MapPartition {
            identity: *identity,
            value: identity.value(),
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        })
        .collect();
    ExecutionMap::try_from_declared_partitions(ids, partitions).unwrap()
}

type Reduced = Result<(u64, ExecutionReport), ReduceInputDenial<()>>;

fn reduce(
    lease: Option<&ExecutionResourceLease<'_>>,
    ceiling: u64,
    count: u64,
) -> (Reduced, ExecutionReport) {
    let input = map(count);
    let computation = || {
        input
            .run_reduce(
                lease,
                |value, context| {
                    context.checkpoint(PARTITION_WORK)?;
                    Ok::<_, MapKernelFailure<()>>(*value)
                },
                0_u64,
                |left: &u64, right: &u64| left + right,
                8,
                0,
            )
            .map(|(tree, report, _)| (*tree.result(), report))
    };
    let ceiling = ExecutionWorkCeiling::new(ceiling);
    match lease {
        Some(lease) => ceiling.run(lease, computation),
        None => ceiling.run_serial(&unbounded(), computation),
    }
    .expect("the computation ran")
}

fn unbounded() -> SerialRequest {
    SerialRequest {
        memory: None,
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

fn exhausted_at(reduced: &Reduced) -> Option<u64> {
    match reduced {
        Err(ReduceInputDenial::MapStopped {
            reason: MapStop::WorkExhausted { identity },
            ..
        }) => Some(identity.value()),
        _ => None,
    }
}

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
        Err(ReduceInputDenial::ReductionStopped { ref failure, .. })
            if matches!(failure.reason, ReductionRunStop::Hook(_))
    ));
}

#[test]
fn the_narrower_of_the_lease_and_the_declared_ceiling_binds() {
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(2).unwrap(),
        charged_memory_bytes: 1 << 20,
    })
    .unwrap();
    let lease = |work| {
        authority
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Automatic,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), 1 << 20, work),
                ),
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .unwrap()
    };
    let (complete, _) = reduce(Some(&lease(10_000)), 10_000, 4);
    let (sum, report) = complete.expect("both bounds admit the work");
    assert_eq!(sum, 10);
    let (declared, _) = reduce(Some(&lease(10_000)), 2 * PARTITION_WORK + 2, 4);
    assert_eq!(exhausted_at(&declared), Some(3));
    let (leased, _) = reduce(Some(&lease(2 * PARTITION_WORK + 2)), 10_000, 4);
    assert_eq!(exhausted_at(&leased), Some(3));
    let (serial, _) = reduce(None, u64::MAX, 4);
    assert_eq!(
        serial.expect("serial completes").1.charged_work(),
        report.charged_work()
    );
}

#[test]
fn a_panic_outside_every_pattern_is_typed_and_the_next_run_is_clean() {
    let panicked = catch_unwind(AssertUnwindSafe(|| {
        ExecutionWorkCeiling::new(10)
            .run_serial(&unbounded(), || -> u64 { panic!("outside a pattern") })
    }))
    .expect("the ceiling contains the panic");
    assert_eq!(panicked.unwrap_err(), WorkCeilingDenial::Panicked);
    let (after, _) = reduce(None, u64::MAX, 2);
    assert_eq!(after.expect("the next run completes").0, 3);
}

#[test]
fn a_serial_request_cancelled_mid_reduction_stops_at_a_combine() {
    let cancellation = CancellationSource::new();
    let request = SerialRequest {
        cancellation: cancellation.token(),
        ..unbounded()
    };
    let input = map(8);
    let combines = std::sync::atomic::AtomicU64::new(0);
    let (reduced, _) = ExecutionWorkCeiling::new(u64::MAX)
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
    let Err(ReduceInputDenial::ReductionStopped { failure, .. }) = reduced else {
        panic!("cancellation stops the reduction");
    };
    assert_eq!(
        failure.reason,
        ReductionRunStop::Hook(MapKernelStop::Cancelled)
    );
    assert_eq!(
        combines.into_inner(),
        1,
        "the next combine boundary observes it"
    );
}

#[test]
fn a_serial_request_past_its_deadline_refuses_before_the_computation() {
    let request = SerialRequest {
        deadline: Some(std::time::Instant::now()),
        ..unbounded()
    };
    let ran = ExecutionWorkCeiling::new(u64::MAX).run_serial(&request, || ());
    assert_eq!(
        ran.unwrap_err(),
        WorkCeilingDenial::Stopped(MapKernelStop::DeadlineElapsed)
    );
}
