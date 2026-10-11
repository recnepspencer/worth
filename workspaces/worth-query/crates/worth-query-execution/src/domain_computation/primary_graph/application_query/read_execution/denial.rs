#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::application_query) enum WorthQueryApplicationReadExecutionDenialKind
{
    ExecutionRequest(crate::domain_computation::WorthQueryProviderSessionDenialKind),
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

    pub(in crate::domain_computation::primary_graph::application_query) fn into_subject(
        self,
    ) -> String {
        self.subject
    }
}

pub(super) fn relational_lookup_denial(
    denial: worth_relational::facade::indexes::BoundedEntityFieldLookupDenial,
    unavailable: WorthQueryApplicationReadExecutionDenialKind,
    subject: &str,
) -> WorthQueryApplicationReadExecutionDenial {
    use worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind as Kind;
    let kind = match denial.kind() {
        Kind::Execution(cause) => return relational_read_denial(cause, subject),
        Kind::InvalidCandidateLimit
        | Kind::SnapshotUnavailable
        | Kind::IndexNotInstalled
        | Kind::WrongIndexKind
        | Kind::ExactGenerationUnavailable
        | Kind::CorruptIndexEntries
        | Kind::StorageParityMismatch => unavailable,
    };
    read_execution_denial(kind, subject)
}

pub(in crate::domain_computation::primary_graph::application_query) fn relational_read_denial(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    subject: &str,
) -> WorthQueryApplicationReadExecutionDenial {
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    let kind = match crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(cause, None) {
        Ok(kind) => WorthQueryApplicationReadExecutionDenialKind::ExecutionRequest(kind),
        Err(Control::Cancelled) => WorthQueryApplicationReadExecutionDenialKind::Cancelled,
        Err(Control::TimedOut) => WorthQueryApplicationReadExecutionDenialKind::DeadlineExceeded,
    };
    read_execution_denial(kind, subject)
}
