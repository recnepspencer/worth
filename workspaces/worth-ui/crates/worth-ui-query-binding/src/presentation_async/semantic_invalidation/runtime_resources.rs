use worth_query::facade::runtime;

const SOURCE_GRAPH_RETAINED_SLOTS: usize = 1;
const SOURCE_GRAPH_RETAINED_BYTES: u64 = 4 * 1024 * 1024;
const SOURCE_GRAPH_CAPTURE_VISITS: usize = 4 * 1024;
const SOURCE_QUERY_CACHE_BYTES: u64 = 12 * 1024;

pub(super) fn presentation_async_resources() -> runtime::WorthQueryConditionalExecutionResources {
    runtime::WorthQueryConditionalExecutionResources::new(
        runtime::WorthQueryConditionalEvaluationCacheBudget::bounded(1, SOURCE_QUERY_CACHE_BYTES)
            .expect("the presentation async runtime retains a positive minimum cache budget"),
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget {
            maximum_retained_slots: SOURCE_GRAPH_RETAINED_SLOTS,
            maximum_retained_bytes: SOURCE_GRAPH_RETAINED_BYTES,
            maximum_attempt_visits: SOURCE_GRAPH_CAPTURE_VISITS,
        },
    )
}
