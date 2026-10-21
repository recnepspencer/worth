use super::*;

pub fn host_limits() -> WorthQueryInMemoryApplicationLimits {
    with_query_resources(runtime::WorthQueryApplicationQueryResourceProfile::new(
        std::num::NonZeroUsize::new(128).expect("concurrent graph work is non-zero"),
    ))
}

pub(in super::super) fn with_query_resources(
    queries: runtime::WorthQueryApplicationQueryResourceProfile,
) -> WorthQueryInMemoryApplicationLimits {
    WorthQueryInMemoryApplicationLimits::new(
        world_resources(256, 4 * 1024 * 1024, 256),
        queries,
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}
