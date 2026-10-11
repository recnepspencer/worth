#[path = "support/test_serialization.rs"]
mod test_serialization;

use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, OnceLock,
    },
};

#[path = "support/reducer_overlap.rs"]
mod reducer_overlap;

use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionMap, LeaseRequest,
    MapKernelFailure, MapPartition, ReduceInputDenial, ReductionPlan, ReductionRunStop,
    ReductionTree,
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
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(8 << 20),
        })
        .unwrap()
    })
}

fn lease(workers: usize) -> worth_execution::ExecutionResourceLease<'static> {
    lease_with_ceiling(workers, 10_000)
}

fn lease_with_ceiling(
    workers: usize,
    work_ceiling: u64,
) -> worth_execution::ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(workers).unwrap(), 8 << 20, work_ceiling),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}

fn map(count: u64) -> ExecutionMap<u64, u64> {
    let ids: Vec<_> = (1..=count).map(PartitionIdentity::new).collect();
    let partitions = ids
        .iter()
        .copied()
        .map(|identity| MapPartition {
            identity,
            value: identity.value(),
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 1,
        })
        .collect();
    ExecutionMap::try_from_declared_partitions(ids, partitions).unwrap()
}

fn mapped_bytes(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<Vec<u8>, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(vec![*value as u8])
}

#[test]
fn frontier_concatenation_preserves_partition_identity_order_at_every_width() {
    let _guard = test_serialization::guard(&TEST_LOCK);
    for workers in [4, 2, 1] {
        let (mut tree, _, _) = map(8)
            .run_reduce(
                Some(&lease(workers)),
                mapped_bytes,
                Vec::<u8>::new(),
                |left, right| left.iter().chain(right).copied().collect(),
                256,
                0,
            )
            .unwrap();
        assert_eq!(
            tree.result(),
            &vec![1, 2, 3, 4, 5, 6, 7, 8],
            "frontier reduction must combine in partition identity order after build"
        );
        tree.update(PartitionIdentity::new(4), vec![40]).unwrap();
        assert_eq!(
            tree.result(),
            &vec![1, 2, 3, 40, 5, 6, 7, 8],
            "frontier reduction must combine in partition identity order after update"
        );
    }
}

fn competing_failures(left: &Vec<u8>, right: &Vec<u8>) -> Vec<u8> {
    if left.is_empty() && right.as_slice() == [4] {
        panic!("later canonical task");
    }
    if left.as_slice() == [1] && right.as_slice() == [2] {
        return vec![9; 64];
    }
    let mut joined = left.clone();
    joined.extend(right);
    joined
}

fn mapped(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<f64, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok((*value as f64).sin())
}

fn sum(left: &f64, right: &f64) -> f64 {
    left + right
}

fn mapped_u64(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<u64, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(*value)
}

fn sum_u64(left: &u64, right: &u64) -> u64 {
    left + right
}

fn prove_frontier(count: u64, workers: usize, expected_overlap: usize) {
    let input = map(count);
    let (oracle, oracle_report, oracle_metrics) = input
        .run_reduce(Some(&lease(1)), mapped, 0.0, sum, 8, 0)
        .unwrap();
    let active = AtomicUsize::new(0);
    let peak = AtomicUsize::new(0);
    let overlap = reducer_overlap::ReducerOverlap::new(expected_overlap);
    let combine = |left: &f64, right: &f64| {
        let now = active.fetch_add(1, Ordering::AcqRel) + 1;
        peak.fetch_max(now, Ordering::AcqRel);
        overlap.rendezvous();
        active.fetch_sub(1, Ordering::AcqRel);
        left + right
    };
    let (native, report, metrics) = input
        .run_reduce(Some(&lease(workers)), mapped, 0.0, combine, 8, 0)
        .unwrap();
    assert_eq!(native.result().to_bits(), oracle.result().to_bits());
    assert_eq!(metrics, oracle_metrics);
    assert_eq!(report.charged_work(), oracle_report.charged_work());
    assert_eq!(report.charged_span(), oracle_report.charged_span());
    assert!(
        peak.load(Ordering::Acquire) >= expected_overlap,
        "expected {expected_overlap} overlapping reducer calls"
    );
}

#[test]
fn four_worker_frontier_and_edge_root_deeper_branch_execute_concurrently() {
    let _guard = test_serialization::guard(&TEST_LOCK);
    prove_frontier(128, 4, 3);
    // Identity 10 has the minimum canonical priority over 1..=10, so the
    // root has one child; the bounded frontier descends to its branches.
    prove_frontier(10, 3, 2);
}

#[test]
fn earlier_join_capacity_failure_precedes_later_task_panic_at_every_width() {
    let _guard = test_serialization::guard(&TEST_LOCK);
    let input = map(7);
    let stopped = |workers| {
        input.run_reduce(
            Some(&lease(workers)),
            mapped_bytes,
            Vec::<u8>::new(),
            competing_failures,
            32,
            0,
        )
    };
    let (serial_failure, serial_report) = match stopped(1) {
        Err(ReduceInputDenial::ReductionStopped { failure, report }) => (failure, report),
        _ => panic!("expected serial reduction failure"),
    };
    let (native_failure, native_report) = match stopped(4) {
        Err(ReduceInputDenial::ReductionStopped { failure, report }) => (failure, report),
        _ => panic!("expected native reduction failure"),
    };
    assert_eq!(
        serial_failure.reason,
        ReductionRunStop::ResultCapacityExceeded
    );
    assert_eq!(native_failure.reason, serial_failure.reason);
    assert_eq!(native_failure.metrics, serial_failure.metrics);
    assert_eq!(native_report.charged_work(), serial_report.charged_work());
    assert_eq!(native_report.charged_span(), serial_report.charged_span());
    assert!(native_report.physical().discarded_in_flight_work() > 0);
}

fn assert_ceiling_width_invariant(input: &ExecutionMap<u64, u64>, count: u64, ceiling: u64) {
    let run = |workers| {
        input.run_reduce(
            Some(&lease_with_ceiling(workers, ceiling)),
            mapped_u64,
            0_u64,
            sum_u64,
            8,
            0,
        )
    };
    let serial = run(1);
    let native = run(4);
    match (serial, native) {
        (
            Err(ReduceInputDenial::ReductionStopped {
                failure: left,
                report: left_report,
            }),
            Err(ReduceInputDenial::ReductionStopped {
                failure: right,
                report: right_report,
            }),
        ) => {
            assert_eq!(
                left.reason,
                ReductionRunStop::Hook(worth_execution::MapKernelStop::WorkCeiling),
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                left_report.charged_work(),
                ceiling,
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                left.metrics.charged_work + count,
                ceiling,
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                left.metrics.charged_span, left.metrics.charged_work,
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                right.reason, left.reason,
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                right.metrics, left.metrics,
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                right_report.charged_work(),
                left_report.charged_work(),
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                right_report.charged_span(),
                left_report.charged_span(),
                "count {count}, ceiling {ceiling}"
            );
        }
        (Ok((left, left_report, left_metrics)), Ok((right, right_report, right_metrics))) => {
            assert_eq!(
                right.result(),
                left.result(),
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                right_metrics, left_metrics,
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                right_report.charged_work(),
                left_report.charged_work(),
                "count {count}, ceiling {ceiling}"
            );
            assert_eq!(
                right_report.charged_span(),
                left_report.charged_span(),
                "count {count}, ceiling {ceiling}"
            );
        }
        _ => panic!("width divergence at count {count}, ceiling {ceiling}"),
    }
}

#[test]
fn work_ceiling_settles_identical_partial_tree_prefix_at_every_width() {
    let _guard = test_serialization::guard(&TEST_LOCK);
    let input = map(7);
    for ceiling in 38..=80 {
        assert_ceiling_width_invariant(&input, 7, ceiling);
    }
    let eleven = map(11);
    assert_ceiling_width_invariant(&eleven, 11, 92);
    let leased_failure = match eleven.run_reduce(
        Some(&lease_with_ceiling(4, 92)),
        mapped_u64,
        0_u64,
        sum_u64,
        8,
        0,
    ) {
        Err(ReduceInputDenial::ReductionStopped { failure, .. }) => failure,
        _ => panic!("expected n=11 reduction ceiling"),
    };
    let identities: Vec<_> = (1..=11).map(PartitionIdentity::new).collect();
    let entries: Vec<_> = identities
        .iter()
        .copied()
        .map(|identity| (identity, identity.value()))
        .collect();
    let plan = ReductionPlan::try_from_sorted_unique(identities).unwrap();
    let mut accepted = 0_u64;
    let oracle_failure =
        match ReductionTree::try_from_declared_checked(plan, entries, 0_u64, sum_u64, 8, || {
            if accepted == 81 {
                return Err(());
            }
            accepted += 1;
            Ok(())
        }) {
            Err(failure) => failure,
            Ok(_) => panic!("expected checked serial oracle ceiling"),
        };
    assert_eq!(leased_failure.metrics, oracle_failure.metrics);
}

#[test]
fn partial_span_is_width_invariant_across_shape_and_frontier_sizes() {
    let _guard = test_serialization::guard(&TEST_LOCK);
    for count in 1..=25 {
        let input = map(count);
        let (_, full_report, _) = input
            .run_reduce(Some(&lease(1)), mapped_u64, 0_u64, sum_u64, 8, 0)
            .unwrap();
        for ceiling in count..=full_report.charged_work() {
            assert_ceiling_width_invariant(&input, count, ceiling);
        }
    }
}
