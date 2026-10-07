use std::num::NonZeroUsize;
use worth_execution::{ExecutionAuthority, ExecutionAuthorityConfig};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub(in crate::domain_computation::primary_graph) fn test_authority() -> &'static ExecutionAuthority
{
    static OWNER: std::sync::OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
    OWNER.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: 1 << 30,
        })
        .unwrap()
    })
}
pub(in crate::domain_computation::primary_graph) fn test_policy(
    workers: NonZeroUsize,
    memory: u64,
) -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Automatic,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(workers, memory, u64::MAX),
    )
}

pub(in crate::domain_computation::primary_graph) fn isolated_request_owner(
) -> std::sync::MutexGuard<'static, ()> {
    static RUN: std::sync::Mutex<()> = std::sync::Mutex::new(());
    RUN.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
