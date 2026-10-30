pub(crate) use {
    crate::charged_bytes_derive::ordinary_context::model::combine,
    crate::charged_bytes_derive::ordinary_context::model::ResidentValue,
};
pub(crate) use {
    std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering,
    std::sync::OnceLock,
};
pub(crate) use {
    worth_execution::CancellationToken, worth_execution::ChargedBytes,
    worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig,
    worth_execution::ExecutionMap, worth_execution::ExecutionResourceLease,
    worth_execution::ExecutionWorkCeiling, worth_execution::LeaseDenial,
    worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapOutcome,
    worth_execution::MapPartition, worth_execution::MapStop,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
    worth_foundational::PartitionIdentity,
};

pub(crate) const BUDGET: u64 = 1 << 20;

pub(crate) fn lease() -> ExecutionResourceLease<'static> {
    static AUTHORITY: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
    AUTHORITY
        .get_or_init(|| {
            worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
                max_workers: std::num::NonZeroUsize::new(2).unwrap(),
                charged_memory_bytes: Some(1 << 28),
            })
            .unwrap()
        })
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::new(2).unwrap(),
                    Some(BUDGET),
                    Some(u64::MAX),
                ),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap()
}

pub(crate) fn map(value: ResidentValue, cap: u64) -> ExecutionMap<ResidentValue, u64> {
    worth_execution::ExecutionMap::try_from_declared_partitions(
        vec![worth_foundational::PartitionIdentity::new(1)],
        vec![MapPartition {
            identity: worth_foundational::PartitionIdentity::new(1),
            value,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: cap,
        }],
    )
    .unwrap()
}
