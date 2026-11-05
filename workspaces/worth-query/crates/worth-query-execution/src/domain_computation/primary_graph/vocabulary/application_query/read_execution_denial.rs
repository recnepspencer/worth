//! Moved whole from `domain_computation::primary_graph::application_query::read_execution::denial`: the items below and every impl they own.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationReadExecutionDenialKind {
    Content(worth_relational::facade::content::RelationalContentDenial),
    ExecutionRequest(crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind),
    PredicateIndexUnavailable,
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
pub struct WorthQueryApplicationReadExecutionDenial {
    kind: WorthQueryApplicationReadExecutionDenialKind,
    subject: String,
}

impl WorthQueryApplicationReadExecutionDenial {
    pub const fn kind(&self) -> WorthQueryApplicationReadExecutionDenialKind {
        self.kind
    }

    pub(in crate::domain_computation::primary_graph) fn subject(&self) -> &str {
        &self.subject
    }

    pub(in crate::domain_computation::primary_graph) fn into_subject(
        self,
    ) -> String {
        self.subject
    }
}
