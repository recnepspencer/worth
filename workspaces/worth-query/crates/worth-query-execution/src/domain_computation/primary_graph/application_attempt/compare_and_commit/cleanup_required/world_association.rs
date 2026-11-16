//! Preserve the actual World subject; distinct owner counters never substitute.
use super::super::WorthQueryApplicationCommitOutcome as Outcome;
use crate::domain_computation::primary_graph::query_obligations::QueryManagedCleanupOriginal;
use worth_runtime_world::facade::CompositePublicationAttemptIdentity;

pub(super) fn from_attempt(
    attempt: &CompositePublicationAttemptIdentity,
) -> Option<QueryManagedCleanupOriginal> {
    attempt.owner_identity().durable().map(|owner| {
        QueryManagedCleanupOriginal::Attempt(
            worth_foundational::facade::DurableCompositePublicationAttemptIdentity::from_recorded(
                owner,
                attempt.ordinal(),
            ),
        )
    })
}
pub(super) fn from_outcome(outcome: &Outcome) -> Option<QueryManagedCleanupOriginal> {
    match outcome {
        Outcome::CleanupRequired(required) => from_outcome(required.outcome()),
        Outcome::ProductUnpublished(original) => from_attempt(original.attempt_identity()),
        Outcome::ProductPreparationPending(pending) => {
            let original = pending.record_handle().identity();
            original.owner_identity().durable().map(|owner| QueryManagedCleanupOriginal::Unpublished(
                worth_foundational::facade::DurableProductUnpublishedOwnerEffectsIdentity::from_recorded(owner, original.ordinal()),
            ))
        }
        Outcome::Committed(receipt) => from_attempt(
            receipt
                .committed_product_publication()
                .publication_attempt(),
        ),
        Outcome::RecoveredCommitted(receipt) => receipt
            .attempt_identity()
            .durable()
            .map(QueryManagedCleanupOriginal::Attempt),
        Outcome::ProductPreparationRefused(_)
        | Outcome::ProductStale(_)
        | Outcome::NoEffect(_)
        | Outcome::OutputAdopted(_)
        | Outcome::Stale(_)
        | Outcome::Cancelled
        | Outcome::TimedOut
        | Outcome::Denied(_)
        | Outcome::Aborted
        | Outcome::Deferred(_)
        | Outcome::Indeterminate(_) => None,
    }
}
