pub use super::*;

pub fn host_limits(
    clock: worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock,
) -> WorthQueryInMemoryApplicationLimits {
    with_query_resources(
        runtime::WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(128).expect("concurrent graph work is non-zero"),
        ),
        clock,
    )
}

pub fn with_query_resources(
    queries: runtime::WorthQueryApplicationQueryResourceProfile,
    clock: worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock,
) -> WorthQueryInMemoryApplicationLimits {
    WorthQueryInMemoryApplicationLimits::new(
        world_resources(256, 4 * 1024 * 1024, 256, clock),
        queries,
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}
