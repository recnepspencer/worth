//! Exercise ordinary ownership journeys with the documented worker-only policy.
use super::*;
pub(super) fn host_limits(
    clock: worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock,
) -> WorthQueryApplicationLimits {
    use worth_query_host::facade::runtime::*;
    let world =
        worth_query_testkit_document_retention::facade::document_world_resources::world_resources(
            256,
            4 * 1024 * 1024,
            256,
            clock,
        )
        .with_execution_policy(ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::new(8).unwrap(),
        ));
    WorthQueryApplicationLimits::new(
        world,
        WorthQueryApplicationQueryResourceProfile::new(std::num::NonZeroUsize::new(128).unwrap()),
        SignalConditionalEvaluationBudget::development(),
    )
}
