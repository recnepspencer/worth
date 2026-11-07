use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

// Bounds this checkpoint fixture World's serial request memory and work.
pub(crate) const CHECKPOINT_EXECUTION_POLICY: ExecutionRequestPolicy = ExecutionRequestPolicy::new(
    ExecutionPosture::Serial,
    DeterminismContract::CanonicalBitwise,
    ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(8_000_000)),
);

// Large authored populations belong to setup's original request, outside the
// World calls whose locality or exhaustion these fixtures measure. This finite
// grant never becomes the installed World's request policy.
pub(crate) const POPULATION_BOOTSTRAP_POLICY: ExecutionRequestPolicy = ExecutionRequestPolicy::new(
    ExecutionPosture::Serial,
    DeterminismContract::CanonicalBitwise,
    ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(16_000_000_000)),
);

/// Metadata capture is setup, separately funded from the installed World policy.
/// Use the existing finite population allowance; this lease owns no worker pool.
pub(crate) fn checkpoint_capture_request(
) -> worth_query_host::facade::runtime::ExecutionResourceLease<'static> {
    worth_query_host::facade::runtime::ExecutionResourceLease::serial(
        worth_query_host::facade::runtime::LeaseRequest {
            policy: POPULATION_BOOTSTRAP_POLICY,
            cancellation: worth_query_host::facade::runtime::CancellationToken::new(),
            deadline: None,
        },
    )
    .expect("the capture owns its original finite setup request")
}
