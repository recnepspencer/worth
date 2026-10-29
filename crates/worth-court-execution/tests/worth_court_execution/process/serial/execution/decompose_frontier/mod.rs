#[allow(unused_imports)] use crate::decompose_frontier::isolated_context::*;
#[allow(unused_imports)]
pub(crate) use {crate::reducer_overlap};
use {std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::OnceLock};

use {worth_execution::BackInput, worth_execution::CancellationToken, worth_execution::DecomposeKernelEditions, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionDecompose, worth_execution::ExecutionMap, worth_execution::InterfaceSolution, worth_execution::InteriorResult, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapPartition};
use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

#[test]
fn leased_full_decomposition_uses_parallel_reduction_frontier() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let input = input(64);
    let ids = (1..=64).map(worth_foundational::PartitionIdentity::new).collect();
    let mut serial = worth_execution::ExecutionDecompose::try_new(
        ids,
        0_u64,
        |left: &u64, right: &u64| left + right,
        1024,
        0,
        8,
        10_000,
        0,
    )
    .unwrap();
    let expected = serial
        .run(Some(&lease(1)), &input, EDITIONS, interior, interface, back)
        .unwrap();

    let active = std::sync::atomic::AtomicUsize::new(0);
    let peak = std::sync::atomic::AtomicUsize::new(0);
    let overlap = crate::reducer_overlap::ReducerOverlap::new(3);
    let combine = |left: &u64, right: &u64| {
        let now = active.fetch_add(1, std::sync::atomic::Ordering::AcqRel) + 1;
        peak.fetch_max(now, std::sync::atomic::Ordering::AcqRel);
        overlap.rendezvous();
        active.fetch_sub(1, std::sync::atomic::Ordering::AcqRel);
        left + right
    };
    let mut native = worth_execution::ExecutionDecompose::try_new(
        (1..=64).map(worth_foundational::PartitionIdentity::new).collect(),
        0_u64,
        combine,
        1024,
        0,
        8,
        10_000,
        0,
    )
    .unwrap();
    let actual = native
        .run(Some(&lease(4)), &input, EDITIONS, interior, interface, back)
        .unwrap();
    assert_eq!(actual.values, expected.values);
    assert_eq!(actual.reduction_metrics, expected.reduction_metrics);
    assert_eq!(
        actual.total_report.charged_work(),
        expected.total_report.charged_work()
    );
    assert_eq!(
        actual.total_report.charged_span(),
        expected.total_report.charged_span()
    );
    assert!(peak.load(std::sync::atomic::Ordering::Acquire) >= 3);
}
