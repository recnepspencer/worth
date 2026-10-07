use super::*;

pub(super) fn resources() -> WorthQueryExecutionResourceContract {
    let scale = worth_query_declaration::facade::domain_computation::WorthQuerySemanticScaleRequest::bounded(64).without_work_budget();
    let resources = WorthQueryExecutionResourceContract::declared([WorthQueryExecutionStrategyContract::new(
        WorthQueryExecutionStrategyName::new("bounded").unwrap(),
        WorthQueryExecutionResourceEnvelope::new(scale,
            worth_query_declaration::facade::domain_computation::WorthQueryResourceLimitRequest::bounded(64),
            WorthQueryExecutionMode::Synchronous, None,
            WorthQueryCancellationSafePointFamily::new("record-boundary").unwrap()),
        WorthQueryExecutionProviderRequirements::new(
            WorthQueryExecutionProviderFamily::new("fixture-provider").unwrap(),
            WorthQueryExecutionAccessProductFamily::new("fixture-access").unwrap(),
            WorthQueryExecutionAllocatorFamily::new("fixture-arena").unwrap()),
    )]).unwrap();
    resources
}
