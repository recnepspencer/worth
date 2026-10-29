#[allow(unused_imports)]
pub(crate) use {crate::test_serialization};

#[allow(unused_imports)]
pub(crate) use {crate::reducer_overlap};
pub(crate) use {std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Mutex, std::sync::OnceLock};

pub(crate) use {worth_execution::CancellationToken, worth_execution::CanonicalBits, worth_execution::ChargedBytes, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapPartition, worth_execution::ReduceInputDenial, worth_execution::ReductionMetrics, worth_execution::ReductionRunFailure, worth_execution::ReductionRunStop};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

pub(crate) static AUTHORITY: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
pub(crate) static TEST_LOCK: Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::new(2).unwrap(),
            charged_memory_bytes: Some(1 << 20),
        })
        .unwrap()
    })
}

pub(crate) fn lease(work: u64) -> worth_execution::ExecutionResourceLease<'static> {
    lease_with_workers(work, 2)
}

pub(crate) fn lease_with_workers(
    work: u64,
    workers: usize,
) -> worth_execution::ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::new(workers).unwrap(),
                    Some(1 << 20),
                    Some(work),
                ),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap()
}

pub(crate) fn map() -> ExecutionMap<u64, u64> {
    map_with_count(3)
}

pub(crate) fn map_with_count(count: u64) -> ExecutionMap<u64, u64> {
    let ids: Vec<_> = (1..=count).map(worth_foundational::PartitionIdentity::new).collect();
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
    worth_execution::ExecutionMap::try_from_declared_partitions(ids, partitions).unwrap()
}

pub(crate) fn mapped(
    value: &u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<u64, MapKernelFailure<()>> {
    context.checkpoint(1)?;
    Ok(*value)
}

pub(crate) fn sum(left: &u64, right: &u64) -> u64 {
    left + right
}

pub(crate) struct PanickingClone(pub(crate) u64);

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