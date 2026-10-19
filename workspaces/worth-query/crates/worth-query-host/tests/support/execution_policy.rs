//! Host policy for Query's serial fixture entries.
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

pub(crate) fn serial_request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    )
}

pub(crate) fn with_bootstrap<R>(
    body: impl for<'scope> FnOnce(
        worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase<
            'scope,
        >,
    ) -> R,
) -> R {
    worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::new(1).unwrap(),
                None,
                None,
            ),
        ),
        body,
    )
    .expect("fixture bootstrap host call opens")
}
