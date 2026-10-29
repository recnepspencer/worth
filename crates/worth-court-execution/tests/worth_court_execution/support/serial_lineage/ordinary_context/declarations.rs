pub(crate) use {std::num::NonZeroUsize};
pub(crate) use {worth_execution::CancellationToken, worth_execution::ExecutionMemoryReservation, worth_execution::ExecutionRequest, worth_execution::LeaseDenial, worth_execution::MemoryLimitDenial, worth_execution::MemoryLimitLevel, worth_execution::SerialRequest};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy};

pub(crate) fn serial(bytes: u64) -> SerialRequest {
    worth_execution::SerialRequest::from_policy(
        &worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(bytes), Some(10_000)),
        ),
        worth_execution::CancellationToken::new(),
        None,
    )
}