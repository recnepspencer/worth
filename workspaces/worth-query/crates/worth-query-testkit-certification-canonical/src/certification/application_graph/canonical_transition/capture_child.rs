//! A transition funded by one lease whose capture a caller-named child caps.
//!
//! The funding lease pays for authoring, the bootstrap commit and the native
//! recovery request. The capture policy is a separate, explicit descendant of
//! that lease, so a court can leave the commit fully funded and still prove
//! that the native ACK precedes a real capture refusal at the child's own
//! zero limit (ancestor 0).
pub use std::num::NonZeroUsize;
pub use worth_execution::{
    CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest,
};
pub use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
pub use worth_query_host::facade::{
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryBootstrapAdvancementPhase,
    },
    application_installation::WorthQueryInMemoryApplicationDenial,
};

pub fn request(
    memory: u64,
    work: u64,
    cancellation: CancellationToken,
    deadline: Option<std::time::Instant>,
) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(memory), Some(work)),
        ),
        deadline,
        cancellation,
    }
}

/// Room for a whole transition; its capture is capped separately.
pub fn funding(
    cancellation: CancellationToken,
    deadline: Option<std::time::Instant>,
) -> LeaseRequest {
    request(64 * 1024 * 1024, 8_000_000, cancellation, deadline)
}

/// Runs one in-advancement transition funded by `funding`.
pub fn funded_by<T>(
    funding: &ExecutionResourceLease<'_>,
    transition: impl FnOnce(
        &WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<T, WorthQueryInMemoryApplicationDenial>,
) -> Result<T, WorthQueryInMemoryApplicationDenial> {
    with_bootstrap_advancement_from_request(ExecutionRequest::leased(funding), |phase| {
        transition(&phase)
    })
    .expect("the funding lease opens the transition")
}
