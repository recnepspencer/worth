//! Bootstrap custody for UI's Query-backed certification worlds.
pub(crate) fn with_bootstrap_request<R>(
    body: impl for<'scope> FnOnce(&worth_runtime_bridge::facade::ExecutionRequest<'scope, 'scope>) -> R,
) -> R {
    worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        crate::query_runtime_resources::UI_EXECUTION_POLICY,
        |phase| body(&phase.execution_request()),
    )
    .expect("UI certification bootstrap host call opens")
}
