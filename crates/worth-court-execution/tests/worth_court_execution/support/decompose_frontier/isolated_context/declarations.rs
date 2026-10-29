#[allow(unused_imports)]
pub(crate) use {crate::reducer_overlap};
pub(crate) use {std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::OnceLock};

pub(crate) use {worth_execution::BackInput, worth_execution::CancellationToken, worth_execution::DecomposeKernelEditions, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionDecompose, worth_execution::ExecutionMap, worth_execution::InterfaceSolution, worth_execution::InteriorResult, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapPartition};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

pub(crate) static AUTHORITY: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
pub(crate) const EDITIONS: DecomposeKernelEditions = DecomposeKernelEditions {
    interior: 1,
    interface: 1,
    back: 1,
};

pub(crate) fn lease(workers: usize) -> worth_execution::ExecutionResourceLease<'static> {
    let authority = AUTHORITY.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(8 << 20),
        })
        .unwrap()
    });
    authority
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::new(workers).unwrap(), None, Some(10_000)),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap()
}

pub(crate) fn input(count: u64) -> ExecutionMap<u64, u64> {
    let ids: Vec<_> = (1..=count).map(worth_foundational::PartitionIdentity::new).collect();
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
    worth_execution::ExecutionMap::try_from_declared_partitions(ids, partitions).unwrap()
}

pub(crate) fn interior(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InteriorResult<u64, u64>, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(InteriorResult {
        interior: *value,
        contribution: *value,
    })
}

pub(crate) fn interface(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<InterfaceSolution<u64, u64>, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(InterfaceSolution {
        solution: *value,
        slices: vec![*value; 64],
    })
}

pub(crate) fn back(
    input: &BackInput<u64, u64>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<u64, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(input.interior + input.interface_slice)
}