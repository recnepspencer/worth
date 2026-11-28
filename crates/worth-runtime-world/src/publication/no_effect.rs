use crate::branch::{ProductBranchObservation, ProductBranchReferenceSnapshot};

/// Typed causes for a no-effect terminal. All of these mean no owner and no
/// product reference moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoEffectCause {
    DurableOpenRequired,
    ExecutionRequest(worth_execution::WorkCeilingDenial),
    StaleExpectedProductHead,
    CancelledBeforeEffect,
    DeadlineBeforeEffect,
    OwnerDeniedBeforeEffect,
    CurrentReadDependency(super::CurrentProductReadDependencyDenial),
    StaleIncarnation,
    CorrespondenceRebindRequired,
    /// Bridge refused the predecessor's exact correspondence before any effect.
    CorrespondenceAdmissionDenied(
        worth_runtime_bridge::facade::RuntimeWorldCorrespondenceAdmissionDenial,
    ),
    ReferenceGenerationExhausted,
    CapacityExhausted,
    OwnerUnavailable,
    RelationalDeferred(worth_relational::facade::mvcc::RelationalPublicationDeferred),
    PreEffectFailure,
}

#[derive(Debug, Clone)]
pub struct NoEffectCompositePublication {
    cause: NoEffectCause,
    expected_head: Option<ProductBranchObservation>,
    observed_head: Option<ProductBranchReferenceSnapshot>,
    native_unique_field_denial: Option<std::sync::Arc<worth_relational::facade::mvcc::RelationalPublicationDenial>>,
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
            native_unique_field_denial: None,
        }
    }

    pub(crate) fn with_observed_head(
        mut self,
        observed_head: Option<ProductBranchReferenceSnapshot>,
    ) -> Self {
        self.observed_head = observed_head;
        self
    }

    pub(crate) fn retain_native_unique_field_denial(
        &mut self,
        denial: Option<std::sync::Arc<worth_relational::facade::mvcc::RelationalPublicationDenial>>,
    ) {
        self.native_unique_field_denial = denial;
    }

    pub fn native_unique_field_denial(&self) -> Option<&worth_relational::facade::mvcc::RelationalPublicationDenial> {
        self.native_unique_field_denial.as_deref()
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
