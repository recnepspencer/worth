//! Recorded envelopes return only authentic original owner outcomes.
use super::*;
pub(super) fn recorded(
    resolution: worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolution,
) -> Result<
    WorthQueryEditEnvelopeOutcome,
    crate::application_entry::WorthQueryPerformedMutationExecutionDenial,
> {
    use worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolution as Resolution;
    Ok(match resolution {
        Resolution::RecoveredCommitted(receipt) => WorthQueryEditEnvelopeOutcome::RecoveredCommitted(receipt),
        Resolution::IntentDrift => WorthQueryEditEnvelopeOutcome::IdempotencyIntentDrift,
        Resolution::ProductUnpublished(owner) => WorthQueryEditEnvelopeOutcome::Uncommitted(WorthQueryApplicationUncommitted::ProductUnpublished(owner)),
        Resolution::ProductPreparationPending(owner) => WorthQueryEditEnvelopeOutcome::Uncommitted(WorthQueryApplicationUncommitted::ProductPreparationPending(owner)),
        Resolution::ReleasedOriginal(_) => return Err(crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Mutation(
            WorthQueryApplicationRequestMutationDenial::Idempotency(worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial::original_effect_released())
        )),
        Resolution::Unseen => unreachable!("Query carries Unseen through its sealed request-local owner"),
    })
}
