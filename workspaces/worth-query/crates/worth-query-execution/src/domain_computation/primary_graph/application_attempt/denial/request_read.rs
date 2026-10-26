//! Native read causes retained at the application attempt boundary.
use super::{
    AttemptDenialCause, AttemptDenialMeaning, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind,
};
impl From<worth_relational::facade::transactions::RelationalExecutionDenialCause>
    for WorthQueryApplicationAttemptDenial
{
    fn from(cause: worth_relational::facade::transactions::RelationalExecutionDenialCause) -> Self {
        Self::relational_read_stopped(cause, "field observation")
    }
}
impl WorthQueryApplicationAttemptDenial {
    pub(in crate::domain_computation::primary_graph) fn native_storage_unavailable(
        cause: worth_relational::facade::storage::RelationalStorageUnavailable,
        subject: &str,
    ) -> Self {
        Self {
            subject: subject.to_owned(),
            meaning: AttemptDenialMeaning::NativeStorage(cause),
        }
    }

    pub fn storage_unavailable(
        &self,
    ) -> Option<&worth_relational::facade::storage::RelationalStorageUnavailable> {
        match &self.meaning {
            AttemptDenialMeaning::NativeStorage(cause) => Some(cause),
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn relational_read_stopped(
        cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
        subject: &str,
    ) -> Self {
        use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
        let kind =
            match crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(
                cause, None,
            ) {
                Ok(kind) => WorthQueryApplicationAttemptDenialKind::ExecutionRequest(kind),
                Err(Control::Cancelled) => WorthQueryApplicationAttemptDenialKind::Cancelled,
                Err(Control::TimedOut) => WorthQueryApplicationAttemptDenialKind::DeadlineExceeded,
            };
        Self::from_cause(
            kind,
            subject.to_owned(),
            AttemptDenialCause::RelationalExecution(cause),
        )
    }

    pub(in crate::domain_computation) fn principal_read_stopped(
        stop: crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial,
        subject: &str,
    ) -> Self {
        use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenialKind as Principal;
        let kind = match stop.kind() {
            Principal::Stopped(Err(
                crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
            )) => WorthQueryApplicationAttemptDenialKind::Cancelled,
            Principal::Stopped(Err(
                crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
            )) => WorthQueryApplicationAttemptDenialKind::DeadlineExceeded,
            Principal::Stopped(Ok(kind)) => {
                WorthQueryApplicationAttemptDenialKind::ExecutionRequest(kind)
            }
            Principal::Refused(_) => {
                WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact
            }
        };
        Self::new(kind, subject)
    }
    pub(in crate::domain_computation) fn request_read_stopped(
        stop: worth_execution::MapKernelStop,
        subject: &str,
    ) -> Self {
        Self::query_read_denial(
            worth_relational::facade::runtime::QueryReadExecutionStop::Request(
                worth_execution::WorkCeilingDenial::Stopped(stop),
            ),
            subject,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn query_read_denial(
        stop: worth_relational::facade::runtime::QueryReadExecutionStop,
        subject: &str,
    ) -> Self {
        use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
        use WorthQueryApplicationAttemptDenialKind as Kind;
        let kind =
            match crate::domain_computation::primary_graph::provider::relational_execution_denial::query_read_execution_kind(
                &stop,
            ) {
                Ok(Some(kind)) => Kind::ExecutionRequest(kind),
                Ok(None) => Kind::MissingAuthoritativeFact,
                Err(Control::Cancelled) => Kind::Cancelled,
                Err(Control::TimedOut) => Kind::DeadlineExceeded,
            };
        Self::from_cause(
            kind,
            subject.to_owned(),
            AttemptDenialCause::NativeQueryRead(Box::new(stop)),
        )
    }
}
