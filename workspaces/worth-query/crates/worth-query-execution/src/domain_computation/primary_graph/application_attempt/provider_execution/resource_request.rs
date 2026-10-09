//! Derivation of the execution-resource request from installed contracts.

pub(in crate::domain_computation) fn application_resource_request(
    contracts: &worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts,
) -> Option<worth_query_declaration::facade::domain_computation::WorthQueryExecutionResourceRequest>
{
    let envelope = contracts.execution_strategy()?.envelope();
    worth_query_declaration::facade::domain_computation::WorthQueryExecutionResourceRequest::atomic(
        envelope.scale_ceilings().clone(),
        envelope.resource_ceilings().clone(),
        envelope.cancellation_safe_point().clone(),
    )
    .ok()
}
