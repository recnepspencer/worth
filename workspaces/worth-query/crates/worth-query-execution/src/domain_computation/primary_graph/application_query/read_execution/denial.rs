#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::application_query) enum WorthQueryApplicationReadExecutionDenialKind
{
    PredicateIndexUnavailable,
    PredicateLookupOverflow,
    TargetIdentityIndexUnavailable,
    TargetIdentityLookupOverflow,
    TargetIdentityNotFound,
    ResultLimitExceeded,
    CardinalityMismatch,
    TraversalUnavailable,
    ContinuationIndexUnavailable,
    ContinuationBoundaryRejected,
    ContinuationGenerationChanged,
    ContinuationPageWidthInvalid,
    ProjectionUnavailable,
    ResultBufferLimitExceeded,
    WorkLimitExceeded,
    Cancelled,
    DeadlineExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryApplicationReadExecutionDenial
{
    kind: WorthQueryApplicationReadExecutionDenialKind,
    subject: String,
}

pub(in crate::domain_computation::primary_graph::application_query) fn read_execution_denial(
    kind: WorthQueryApplicationReadExecutionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationReadExecutionDenial {
    WorthQueryApplicationReadExecutionDenial {
        kind,
        subject: subject.into(),
    }
}

impl WorthQueryApplicationReadExecutionDenial {
    pub(in crate::domain_computation::primary_graph::application_query) const fn kind(
        &self,
    ) -> WorthQueryApplicationReadExecutionDenialKind {
        self.kind
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn subject(&self) -> &str {
        &self.subject
    }
}
