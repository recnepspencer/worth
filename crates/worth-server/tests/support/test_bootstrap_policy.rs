//! Bootstrap phase policy for Server fixture installation.
pub(crate) fn with_bootstrap<R>(
    body: impl for<'scope> FnOnce(&worth_execution::ExecutionRequest<'scope, 'scope>) -> R,
) -> R {
    worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                64 * 1024 * 1024,
                None,
            ),
        ),
        |phase| body(&phase.execution_request()),
    )
    .expect("fixture initial installation opens its bootstrap phase")
}
