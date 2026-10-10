use std::{num::NonZeroUsize, sync::OnceLock};
use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionResourceLease,
    LeaseRequest,
};
use worth_foundational::ExecutionRequestPolicy;

fn test_execution_authority() -> &'static ExecutionAuthority {
    static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(64 * 1024 * 1024),
        })
        .expect("one declared Bridge test host")
    })
}

pub(crate) fn test_execution_lease_for_policy(
    policy: ExecutionRequestPolicy,
    cancellation: CancellationToken,
) -> ExecutionResourceLease<'static> {
    test_execution_authority()
        .request_lease(LeaseRequest {
            policy,
            cancellation,
            deadline: None,
        })
        .expect("declared leased test policy")
}

pub(crate) fn test_execution_lease(
    cancellation: worth_execution::CancellationToken,
) -> worth_execution::ExecutionResourceLease<'static> {
    use std::num::NonZeroUsize;
    use worth_execution::LeaseRequest;
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };

    let authority = test_execution_authority();
    authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), 4096, 10),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap()
}
