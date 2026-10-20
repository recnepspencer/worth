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
    // Installation admits the binding's maximum publication shape even though
    // the ordinary Document handler requests its narrow candidate at execution.
    WorthQueryInMemoryApplicationLimits::new(
        world_resources(256, 4 * 1024 * 1024, 256),
        runtime::WorthQueryApplicationCandidateResourceProfile::policy(
            Some(200_000),
            Some(128 * 1024 * 1024),
        )
        .expect("valid candidate limits"),
        queries,
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}
