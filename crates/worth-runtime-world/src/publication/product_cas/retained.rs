use crate::branch::ProductBranchReferenceSnapshot;
use crate::publication::custody::RetainedCommitDisposition;
use crate::recovery::ProductUnpublishedCause;

use super::{CompositePublicationReadyInputs, RuntimeWorldPublicationOutcome};

pub(super) fn retain_before_product_movement(
    ready: CompositePublicationReadyInputs,
    observed: ProductBranchReferenceSnapshot,
    cause: ProductUnpublishedCause,
) -> RuntimeWorldPublicationOutcome {
    RuntimeWorldPublicationOutcome::ProductUnpublished(ready.custody.retain(
        cause,
        Some(observed),
        RetainedCommitDisposition::ReleaseUnused,
    ))
}
