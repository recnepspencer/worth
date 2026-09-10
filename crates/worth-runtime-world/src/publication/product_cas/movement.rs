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
                RetainedCommitDisposition::ReleaseUnused,
            ))
        }
        Err(crate::publication::custody::AttemptProductMovementFailure::Reference(loss)) => {
            let cause = match loss.cutoff_denial() {
                Some(crate::publication::ProductMovementCutoffDenial::Cancelled) => {
                    ready.counters.record_cancellation_observation();
                    ProductUnpublishedCause::CancellationAfterEffect
                }
                Some(crate::publication::ProductMovementCutoffDenial::Deadline) => {
                    ProductUnpublishedCause::DeadlineAfterEffect
                }
                None => ProductUnpublishedCause::ProductPublicationLost,
            };
            RuntimeWorldPublicationOutcome::ProductUnpublished(ready.custody.retain(
                cause,
                Some(loss.observed_head().clone()),
                RetainedCommitDisposition::ReleaseUnused,
            ))
        }
    }
}
