use crate::domain_computation::primary_graph::vocabulary::application_query::read_execution_denial::WorthQueryApplicationReadExecutionDenial;
use crate::domain_computation::primary_graph::vocabulary::application_query::read_execution_denial::WorthQueryApplicationReadExecutionDenialKind;

pub(in crate::domain_computation::primary_graph::application_query) fn read_execution_denial(
    kind: WorthQueryApplicationReadExecutionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationReadExecutionDenial {
    WorthQueryApplicationReadExecutionDenial {
        kind,
        subject: subject.into(),
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
    subject: impl Into<String>,
) -> WorthQueryApplicationReadExecutionDenial {
    use crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind as Control;
    let kind = match crate::domain_computation::primary_graph::vocabulary::provider::relational_execution_denial::relational_execution_kind(cause, None) {
        Ok(kind) => WorthQueryApplicationReadExecutionDenialKind::ExecutionRequest(kind),
        Err(Control::Cancelled) => WorthQueryApplicationReadExecutionDenialKind::Cancelled,
        Err(Control::TimedOut) => WorthQueryApplicationReadExecutionDenialKind::DeadlineExceeded,
    };
    read_execution_denial(kind, subject)
}

/// Both native pinning and prepared execution forward the complete managed stop.
/// Interruption remains a map stop instead of becoming an application read error.
pub(in crate::domain_computation::primary_graph::application_query) fn managed_read_denial(
    cause: crate::domain_computation::provider_session::WorthQueryManagedGraphReadDenial,
    plan: &super::read_plan::ReadPlan<'_>,
) -> Result<
    crate::domain_computation::primary_graph::vocabulary::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial,
    worth_execution::MapKernelStop,
>{
    use crate::domain_computation::primary_graph::vocabulary::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial as Denial;
    use crate::domain_computation::provider_session::WorthQueryManagedGraphReadDenial as Cause;
    match cause {
        Cause::SnapshotUnavailable => Ok(Denial::Read(plan.snapshot_unavailable_denial())),
        Cause::IndexNotInstalled(_) => Ok(Denial::Read(plan.predicate_unavailable_denial())),
        Cause::RequestStop(cause) => match cause.as_kernel_stop() {
            Some(stop) => Err(stop),
            None => Ok(Denial::Read(relational_read_denial(cause, plan.name))),
        },
        Cause::MutationSession
        | Cause::ForeignBasis
        | Cause::ForeignGraph
        | Cause::ForeignReadProof
        | Cause::ForeignReadIntent => Ok(Denial::Session),
    }
}

pub(super) fn content_read_denial(
    cause: worth_relational::facade::content::RelationalContentDenial,
    subject: &str,
) -> WorthQueryApplicationReadExecutionDenial {
    match cause {
        worth_relational::facade::content::RelationalContentDenial::Execution(cause) => {
            relational_read_denial(cause, subject)
        }
        cause => read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::Content(cause),
            subject,
        ),
    }
}

#[cfg(test)]
mod content_tests;
