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
        Kind::Execution(cause) => return relational_cause_denial(cause, subject),
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

pub(super) fn relational_cause_denial(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    subject: impl Into<String>,
) -> WorthQueryApplicationReadExecutionDenial {
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    let kind = match crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(cause, None) {
        Ok(kind) => WorthQueryApplicationReadExecutionDenialKind::ExecutionRequest(kind),
        Err(Control::Cancelled) => WorthQueryApplicationReadExecutionDenialKind::Cancelled,
        Err(Control::TimedOut) => WorthQueryApplicationReadExecutionDenialKind::DeadlineExceeded,
    };
    read_execution_denial(kind, subject)
}

pub(in crate::domain_computation::primary_graph::application_query) fn managed_read_denial(
    cause: crate::domain_computation::provider_session::WorthQueryManagedGraphReadDenial,
    subject: &str,
) -> super::super::authorized_read::WorthQueryAuthorizedApplicationReadDenial {
    use super::super::authorized_read::WorthQueryAuthorizedApplicationReadDenial as Denial;
    use crate::domain_computation::provider_session::WorthQueryManagedGraphReadDenial as Cause;
    match cause {
        Cause::SnapshotUnavailable => Denial::Read(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
            subject,
        )),
        Cause::IndexNotInstalled(_) => Denial::Read(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::PredicateIndexUnavailable,
            subject,
        )),
        Cause::RequestStop(stop) => Denial::Read(relational_cause_denial(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::from(stop),
            subject,
        )),
        Cause::MutationSession
        | Cause::ForeignBasis
        | Cause::ForeignGraph
        | Cause::ForeignReadProof
        | Cause::TerminalReleaseMismatch => Denial::Session,
    }
}
