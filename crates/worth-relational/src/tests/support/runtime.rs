//! The shared SDK execution authority for Relational unit tests.
use std::{num::NonZeroUsize, sync::OnceLock};
use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionResourceLease,
    LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

static TEST_EXECUTION_AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();

pub(crate) fn test_execution_authority() -> &'static ExecutionAuthority {
    TEST_EXECUTION_AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(8).expect("positive test worker count"),
            charged_memory_bytes: Some(512 * 1024 * 1024),
        })
        .expect("one execution authority for relational lib tests")
    })
}

pub(crate) fn test_execution_lease() -> ExecutionResourceLease<'static> {
    test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    NonZeroUsize::new(4).expect("positive test worker count"),
                    None,
                    Some(1_000_000),
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .expect("test execution lease admitted")
}
