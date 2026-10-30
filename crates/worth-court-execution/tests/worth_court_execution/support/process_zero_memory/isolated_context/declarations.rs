// Separate process because the physical authority is constructed once per process.
pub(crate) use std::num::NonZeroUsize;
pub(crate) use {
    worth_execution::CancellationToken, worth_execution::ExecutionAllocationDenialKind,
    worth_execution::ExecutionAllocationPolicy, worth_execution::ExecutionAuthority,
    worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionByteBuffer,
    worth_execution::LeaseDenial, worth_execution::LeaseRequest,
    worth_execution::MemoryLimitDenial, worth_execution::MemoryLimitLevel,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
};
pub(crate) fn request(bytes: u64) -> LeaseRequest {
    LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::new(1).unwrap(),
                Some(bytes),
                Some(1),
            ),
        ),
        deadline: None,
        cancellation: worth_execution::CancellationToken::new(),
    }
}
