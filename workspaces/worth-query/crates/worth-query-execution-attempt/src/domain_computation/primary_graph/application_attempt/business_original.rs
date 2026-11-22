pub use worth_query_execution_lineage::facade::primary_graph::application_attempt::business_original::*;
use crate::domain_computation::primary_graph::query_obligations::QueryManagedCleanupOriginal;

pub fn from_progression(
    outcome: &super::provider_execution::WorthQueryProviderProgressionOutcome,
) -> Option<QueryManagedCleanupOriginal> {
    use super::provider_execution::WorthQueryProviderProgressionOutcome as Progression;
    match outcome {
        Progression::Committed(receipt) => {
            from_attempt(receipt.original_publication().publication_attempt())
        }
        Progression::RecoveredCommitted(receipt) => receipt
            .attempt_identity()
            .durable()
            .map(QueryManagedCleanupOriginal::Attempt),
        Progression::ProductUnpublished(original) => from_attempt(original.attempt_identity()),
        Progression::ProductPreparationPending(pending) => from_preparation(pending),
        Progression::CoupledFailure(outcome) => from_uncommitted(outcome),
        Progression::ProductPreparationRefused(_)
        | Progression::ProductStale(_)
        | Progression::NoEffect(_)
        | Progression::OutputAdopted(_)
        | Progression::Stale(_)
        | Progression::Cancelled
        | Progression::TimedOut
        | Progression::Denied(_)
        | Progression::Aborted
        | Progression::Deferred(_)
        | Progression::Indeterminate(_) => None,
    }
}
fn from_preparation(
    pending: &crate::domain_computation::WorthQueryProductPreparationPending,
) -> Option<QueryManagedCleanupOriginal> {
    let original = pending.record_handle().identity();
    original.owner_identity().durable().map(|owner| QueryManagedCleanupOriginal::Unpublished(
        worth_foundational::facade::DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(owner, original.ordinal()),
    ))
}
fn from_uncommitted(
    outcome: &super::WorthQueryApplicationUncommitted,
) -> Option<QueryManagedCleanupOriginal> {
    use super::WorthQueryApplicationUncommitted as Uncommitted;
    match outcome {
        Uncommitted::CleanupRequired(required) => from_outcome(required.outcome()),
        Uncommitted::ProductUnpublished(original) => from_attempt(original.attempt_identity()),
        Uncommitted::ProductPreparationPending(pending) => from_preparation(pending),
        Uncommitted::ProductPreparationRefused(_)
        | Uncommitted::ProductStale(_)
        | Uncommitted::NoEffect(_)
        | Uncommitted::OutputAdopted(_)
        | Uncommitted::Stale(_)
        | Uncommitted::Cancelled
        | Uncommitted::TimedOut
        | Uncommitted::Denied(_)
        | Uncommitted::Aborted
        | Uncommitted::Deferred(_)
        | Uncommitted::Indeterminate(_) => None,
    }
}
