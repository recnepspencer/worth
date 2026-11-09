//! Numeric-correction requests use the same resources in serial and worker-only courts.
use super::*;
pub(in super::super) fn installation_limits() -> WorthQueryInMemoryApplicationLimits {
    limits(false)
}
pub(in super::super) fn worker_limits() -> WorthQueryInMemoryApplicationLimits {
    limits(true)
}
fn limits(worker: bool) -> WorthQueryInMemoryApplicationLimits {
    use worth_query_host::facade::runtime::*;
    let world = world_resources::world_resources(256, 4 * 1024 * 1024, 256,
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let world = if worker {
        world.with_execution_policy(ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::new(8).unwrap(),
        ))
    } else {
        world
    };
    WorthQueryInMemoryApplicationLimits::new(
        world,
        WorthQueryApplicationQueryResourceProfile::new(std::num::NonZeroUsize::new(128).unwrap()),
        SignalConditionalEvaluationBudget::development(),
    )
}
