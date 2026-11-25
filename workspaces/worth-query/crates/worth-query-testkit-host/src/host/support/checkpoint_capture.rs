//! Bounded execution resource leases for Directory Home host tests.
pub use std::num::NonZeroUsize;
pub use worth_execution::{
    CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest,
};
pub use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub use worth_query_host::facade::application_contribution::{
    with_bootstrap_advancement_from_request, WorthQueryBootstrapAdvancementPhase,
};

pub fn lease(bytes: u64) -> ExecutionResourceLease<'static> {
    bounded_lease(bytes, CancellationToken::new())
}

pub fn bounded_lease(
    bytes: u64,
    cancellation: CancellationToken,
) -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(bytes), None),
        ),
        cancellation,
        deadline: None,
    })
    .unwrap()
}
