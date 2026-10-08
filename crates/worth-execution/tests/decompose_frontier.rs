use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        OnceLock,
    },
    time::Duration,
};

use worth_execution::{
    BackInput, CancellationToken, DecomposeKernelEditions, ExecutionAuthority,
    ExecutionAuthorityConfig, ExecutionDecompose, ExecutionMap, InterfaceSolution, InteriorResult,
    LeaseRequest, MapKernelFailure, MapPartition,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
const EDITIONS: DecomposeKernelEditions = DecomposeKernelEditions {
    interior: 1,
    interface: 1,
    back: 1,
};

fn lease(workers: usize) -> worth_execution::ExecutionResourceLease<'static> {
    let authority = AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(8 << 20),
        })
        .unwrap()
    });
    authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(workers).unwrap(), 8 << 20, 10_000),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}

fn input(count: u64) -> ExecutionMap<u64, u64> {
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
            max_result_bytes: 0,
        })
        .collect();
    ExecutionMap::try_from_declared_partitions(ids, partitions).unwrap()
}

fn interior(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InteriorResult<u64, u64>, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(InteriorResult {
        interior: *value,
        contribution: *value,
    })
}

fn interface(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InterfaceSolution<u64, u64>, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(InterfaceSolution {
        solution: *value,
        slices: vec![*value; 64],
    })
}

fn back(
    input: &BackInput<u64, u64>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<u64, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(input.interior + input.interface_slice)
}

#[test]
fn leased_full_decomposition_uses_parallel_reduction_frontier() {
    let input = input(64);
    let ids = (1..=64).map(PartitionIdentity::new).collect();
    let mut serial = ExecutionDecompose::try_new(
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

    let active = AtomicUsize::new(0);
    let peak = AtomicUsize::new(0);
    let combine = |left: &u64, right: &u64| {
        let now = active.fetch_add(1, Ordering::AcqRel) + 1;
        peak.fetch_max(now, Ordering::AcqRel);
        std::thread::sleep(Duration::from_millis(3));
        active.fetch_sub(1, Ordering::AcqRel);
        left + right
    };
    let mut native = ExecutionDecompose::try_new(
        (1..=64).map(PartitionIdentity::new).collect(),
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
    assert!(peak.load(Ordering::Acquire) >= 3);
}
