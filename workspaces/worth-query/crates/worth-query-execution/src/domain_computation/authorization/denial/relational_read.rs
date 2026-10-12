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

pub(in crate::domain_computation) fn principal_currentness(
    denied: crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial,
) -> Kind {
    use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenialKind as Principal;
    match denied.kind() {
        Principal::ExecutionRequest(kind) => Kind::ExecutionRequest(kind),
        Principal::Cancelled => Kind::Cancelled,
        Principal::DeadlineExceeded => Kind::DeadlineExceeded,
        _ => Kind::StalePrincipal,
    }
}

pub(in crate::domain_computation::authorization) fn kernel_stop_kind(
    stop: worth_execution::MapKernelStop,
) -> Kind {
    use worth_execution::MapKernelStop as Stop;
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
    let cause = match stop {
        Stop::Cancelled => Cause::Cancelled,
        Stop::DeadlineElapsed => Cause::DeadlineElapsed,
        Stop::WorkCeiling => Cause::WorkExhausted,
        Stop::WorkCounterOverflow => Cause::WorkCounterOverflow,
        Stop::NestedStopped => Cause::NestedStopped,
    };
    denial(cause, "authorization request").kind()
}

pub(in crate::domain_computation::authorization) const fn is_request_stop(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::ExecutionRequest(_) | Kind::Cancelled | Kind::DeadlineExceeded
    )
}
