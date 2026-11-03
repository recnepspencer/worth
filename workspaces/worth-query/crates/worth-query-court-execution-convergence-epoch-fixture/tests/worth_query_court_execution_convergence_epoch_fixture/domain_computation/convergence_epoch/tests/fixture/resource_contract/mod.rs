use worth_query_installation::facade::WorthQueryExecutionProviderRequirements;
use worth_query_installation::facade::WorthQueryExecutionResourceContract;
use worth_query_installation::facade::WorthQueryExecutionStrategyContract;
use worth_query_installation::facade::WorthQueryExecutionStrategyName;

pub fn resource_contract(
    support: &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport,
) -> WorthQueryExecutionResourceContract {
    WorthQueryExecutionResourceContract::declared([WorthQueryExecutionStrategyContract::new(
        WorthQueryExecutionStrategyName::new("convergence-bounded").unwrap(),
        support.envelope().clone(),
        WorthQueryExecutionProviderRequirements::new(
            support.provider().clone(),
            support.access_product().clone(),
            support.allocator().clone(),
        ),
    )])
    .expect("fixture resource contract must validate")
}
