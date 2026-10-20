use super::*;

pub fn host_limits() -> WorthQueryInMemoryApplicationLimits {
    with_query_resources(
        runtime::WorthQueryApplicationQueryResourceProfile::bounded(5_120, 2_048, usize::MAX, 128)
            .expect("valid query limits"),
    )
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
