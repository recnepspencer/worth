use crate::branch::ProductBranchReferenceCell;
use crate::publication::custody::RetainedCommitDisposition;
use crate::recovery::ProductUnpublishedCause;

use super::super::{CompositeLateCancellationPosture, RuntimeWorldPublicationOutcome};
use super::CompositePublicationReadyInputs;

pub(super) fn attempt_product_movement(
    mut ready: CompositePublicationReadyInputs,
    cell: &ProductBranchReferenceCell,
    late: CompositeLateCancellationPosture,
    cutoff: Option<crate::publication::ProductMovementCutoff>,
) -> RuntimeWorldPublicationOutcome {
    let disposition = if ready.custody.has_durable_open() {
        RetainedCommitDisposition::KeepOriginal
    } else {
        RetainedCommitDisposition::ReleaseUnused
    };
    match ready.custody.attempt_movement(
        &ready.expected_head,
        &ready.commit,
        &ready.owner_results,
        &mut ready.counters,
        late,
        cell,
        cutoff,
    ) {
        Ok(performed) => RuntimeWorldPublicationOutcome::Performed(performed),
        Err(crate::publication::custody::AttemptProductMovementFailure::Observation(denial)) => {
            let cause = ProductUnpublishedCause::from_retention_denial(&denial);
            RuntimeWorldPublicationOutcome::ProductUnpublished(ready.custody.retain(
                cause,
                None,
                disposition,
            ))
        }
        Err(crate::publication::custody::AttemptProductMovementFailure::Reference(loss)) => {
            let cause = match (loss.is_retired(), loss.cutoff_denial()) {
                (true, _) => ProductUnpublishedCause::StaleProductHead,
                (false, Some(crate::publication::ProductMovementCutoffDenial::Cancelled)) => {
                    ready.counters.record_cancellation_observation();
                    ProductUnpublishedCause::CancellationAfterEffect
                }
                (false, Some(crate::publication::ProductMovementCutoffDenial::Deadline)) => {
                    ProductUnpublishedCause::DeadlineAfterEffect
                }
                (false, None) => ProductUnpublishedCause::ProductPublicationLost,
            };
            RuntimeWorldPublicationOutcome::ProductUnpublished(ready.custody.retain(
                cause,
                Some(loss.observed_head().clone()),
                disposition,
            ))
        }
    }
}
