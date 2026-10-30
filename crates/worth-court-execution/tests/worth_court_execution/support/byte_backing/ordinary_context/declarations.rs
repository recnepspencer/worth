// Actual sealed payload custody. Allocator/header/ledger metadata is uncharged.
#[allow(unused_imports)]
pub(crate) use crate::test_serialization;

pub(crate) use {
    std::num::NonZeroUsize, std::sync::Arc, std::sync::Mutex, std::sync::OnceLock,
    std::time::Duration, std::time::Instant,
};
pub(crate) use {
    worth_execution::CancellationSource, worth_execution::CancellationToken,
    worth_execution::ExecutionAllocationDenialClass as Kind,
    worth_execution::ExecutionAllocationPolicy as Policy, worth_execution::ExecutionAuthority,
    worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionByteBuffer,
    worth_execution::ExecutionImmutableBytes, worth_execution::LeaseRequest,
    worth_execution::MemoryLimitDenial, worth_execution::MemoryLimitLevel,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
};

pub(crate) static AUTHORITY: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
pub(crate) static SERIAL: Mutex<()> = std::sync::Mutex::new(());
pub(crate) fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::new(1).unwrap(),
            charged_memory_bytes: None,
        })
        .unwrap()
    })
}
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
