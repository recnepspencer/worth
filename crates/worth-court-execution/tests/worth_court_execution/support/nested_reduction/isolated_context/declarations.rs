// Nested reducer ownership through the public request and map doors.
pub(crate) use worth_execution::*;
pub(crate) use {std::num::NonZeroUsize, std::sync::Mutex, std::sync::OnceLock};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
    worth_foundational::PartitionIdentity,
};

pub(crate) const MEMORY: u64 = 1 << 20;
pub(crate) static TEST_LOCK: Mutex<()> = std::sync::Mutex::new(());
pub(crate) static AUTHORITY: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
pub(crate) fn request(width: usize) -> ExecutionResourceLease<'static> {
    AUTHORITY
        .get_or_init(|| {
            ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
                max_workers: std::num::NonZeroUsize::new(4).unwrap(),
                charged_memory_bytes: Some(MEMORY),
            })
            .unwrap()
        })
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::new(width).unwrap(),
                    Some(MEMORY),
                    Some(100),
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap()
}
pub(crate) fn singleton() -> ExecutionMap<u64, u64> {
    ExecutionMap::from_keyless_partitions(std::collections::BTreeMap::from([(
        worth_foundational::PartitionIdentity::new(1),
        KeylessPartition {
            value: 1_u64,
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        },
    )]))
    .unwrap()
}
