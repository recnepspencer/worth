#[allow(unused_imports)] use crate::reduce_frontier::isolated_context::*;
#[allow(unused_imports)]
pub(crate) use {crate::test_serialization};

#[allow(unused_imports)]
pub(crate) use {crate::reducer_overlap};
use {std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Mutex, std::sync::OnceLock};

use {worth_execution::CancellationToken, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapPartition, worth_execution::ReduceInputDenial, worth_execution::ReductionPlan, worth_execution::ReductionRunStop, worth_execution::ReductionTree};
use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

#[test]
fn frontier_concatenation_preserves_partition_identity_order_at_every_width() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = crate::test_serialization::guard(&TEST_LOCK);
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
        tree.update(worth_foundational::PartitionIdentity::new(4), vec![40]).unwrap();
        assert_eq!(
            tree.result(),
            &vec![1, 2, 3, 40, 5, 6, 7, 8],
            "frontier reduction must combine in partition identity order after update"
        );
    }
}

#[test]
fn four_worker_frontier_and_edge_root_deeper_branch_execute_concurrently() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = crate::test_serialization::guard(&TEST_LOCK);
    prove_frontier(128, 4, 3);
    // Identity 10 has the minimum canonical priority over 1..=10, so the
    // root has one child; the bounded frontier descends to its branches.
    prove_frontier(10, 3, 2);
}

#[test]
fn earlier_join_capacity_failure_precedes_later_task_panic_at_every_width() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = crate::test_serialization::guard(&TEST_LOCK);
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
        Err(worth_execution::ReduceInputDenial::ReductionStopped { failure, report }) => (failure, report),
        _ => panic!("expected serial reduction failure"),
    };
    let (native_failure, native_report) = match stopped(4) {
        Err(worth_execution::ReduceInputDenial::ReductionStopped { failure, report }) => (failure, report),
        _ => panic!("expected native reduction failure"),
    };
    assert_eq!(
        serial_failure.reason,
        worth_execution::ReductionRunStop::ResultCapacityExceeded
    );
    assert_eq!(native_failure.reason, serial_failure.reason);
    assert_eq!(native_failure.metrics, serial_failure.metrics);
    assert_eq!(native_report.charged_work(), serial_report.charged_work());
    assert_eq!(native_report.charged_span(), serial_report.charged_span());
    assert!(native_report.physical().discarded_in_flight_work() > 0);
}

#[test]
fn work_ceiling_settles_identical_partial_tree_prefix_at_every_width() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = crate::test_serialization::guard(&TEST_LOCK);
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
        Err(worth_execution::ReduceInputDenial::ReductionStopped { failure, .. }) => failure,
        _ => panic!("expected n=11 reduction ceiling"),
    };
    let identities: Vec<_> = (1..=11).map(worth_foundational::PartitionIdentity::new).collect();
    let entries: Vec<_> = identities
        .iter()
        .copied()
        .map(|identity| (identity, identity.value()))
        .collect();
    let plan = worth_execution::ReductionPlan::try_from_sorted_unique(identities).unwrap();
    let mut accepted = 0_u64;
    let oracle_failure =
        match worth_execution::ReductionTree::try_from_declared_checked(plan, entries, 0_u64, sum_u64, 8, || {
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
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = crate::test_serialization::guard(&TEST_LOCK);
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

#[path = "cancelled_prefix/mod.rs"]
pub(crate) mod cancelled_prefix;
