pub(crate) fn with_bootstrap_request<R>(
    body: impl for<'scope> FnOnce(&worth_runtime_bridge::facade::ExecutionRequest<'scope, 'scope>) -> R,
) -> R {
    use worth_query::facade::runtime::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            Some(128 * 1024 * 1024),
            Some(16_000_000),
        ),
    );
    worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        policy,
        |phase| body(&phase.execution_request()),
    )
    .expect("certification bootstrap host call opens")
}
