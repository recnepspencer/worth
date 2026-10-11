//! Native read causes retained at the application attempt boundary.
use super::{
    AttemptDenialCause, WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
impl WorthQueryApplicationAttemptDenial {
    pub(in crate::domain_computation::primary_graph) fn relational_read_stopped(
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
        Self {
            kind,
            subject: subject.to_owned(),
            cause: AttemptDenialCause::NativeQueryRead(Box::new(stop)),
        }
    }
}
