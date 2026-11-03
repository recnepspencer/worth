//! Existing memory publication uses the shared one-time Native mechanics.
use super::*;
mod publication;
pub(super) fn commit(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    mut query: durable_custody::QueryPreparingApplicationCommit,
    mint: super::super::WorthQueryCommitProgressionMint,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    WorthQueryCommittedApplicationSession,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let _ = mint;
    if let Err(cause) = native_work::prepare_native_candidate(&execution, provider, &mut query) {
        return Err(native_work::memory_stop(cause, &query.attempt));
    }
    if let Err(cause) =
        native_work::prepare_candidate_support(&execution, provider, &mut query, allocation_policy)
    {
        return Err(native_work::memory_stop(cause, &query.attempt));
    }
    publication::publish(execution, provider, query)
}
