use crate::branch::{ProductBranchObservation, ProductBranchReferenceSnapshot};

/// Typed causes for a no-effect terminal. All of these mean no owner and no
/// product reference moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoEffectCause {
    StaleExpectedProductHead,
    CancelledBeforeEffect,
    DeadlineBeforeEffect,
    OwnerDeniedBeforeEffect,
    CorrespondenceRebindRequired,
    ReferenceGenerationExhausted,
    CapacityExhausted,
    OwnerUnavailable,
    PreEffectFailure,
}

#[derive(Debug, Clone)]
pub struct NoEffectCompositePublication {
    cause: NoEffectCause,
    expected_head: Option<ProductBranchObservation>,
    observed_head: Option<ProductBranchReferenceSnapshot>,
}

impl NoEffectCompositePublication {
    pub(crate) fn new(
        cause: NoEffectCause,
        expected_head: Option<ProductBranchObservation>,
    ) -> Self {
        Self {
            cause,
            expected_head,
            observed_head: None,
        }
    }

    pub(crate) fn with_observed_head(
        mut self,
        observed_head: Option<ProductBranchReferenceSnapshot>,
    ) -> Self {
        self.observed_head = observed_head;
        self
    }

    pub const fn cause(&self) -> NoEffectCause {
        self.cause
    }

    pub fn expected_head(&self) -> Option<&ProductBranchObservation> {
        self.expected_head.as_ref()
    }

    pub fn observed_head(&self) -> Option<&ProductBranchReferenceSnapshot> {
        self.observed_head.as_ref()
    }
}
