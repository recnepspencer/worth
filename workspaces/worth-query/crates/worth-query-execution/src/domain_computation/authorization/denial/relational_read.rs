//! Request refusals at authorization's Relational read boundary.
use super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind as Kind,
};
pub(in crate::domain_computation::authorization) fn denial(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    subject: &str,
) -> WorthQueryOperationAuthorizationDenial {
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    let kind = match crate::facade::primary_graph::relational_execution_kind(cause, None) {
        Ok(kind) => Kind::ExecutionRequest(kind),
        Err(Control::Cancelled) => Kind::Cancelled,
        Err(Control::TimedOut) => Kind::DeadlineExceeded,
    };
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
