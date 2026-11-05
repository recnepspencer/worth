//! Request refusals at authorization's Relational read boundary.
use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use {
    super::WorthQueryOperationAuthorizationDenial,
    crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind as Kind,
};
pub(in crate::domain_computation::authorization) fn denial(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    subject: &str,
) -> WorthQueryOperationAuthorizationDenial {
    use crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind as Control;
    let kind = match crate::facade::primary_graph::relational_execution_kind(cause, None) {
        Ok(kind) => Kind::Stopped(Ok(kind)),
        Err(Control::Cancelled) => Kind::Stopped(Err(
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::Cancelled,
        )),
        Err(Control::TimedOut) => Kind::Stopped(Err(
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::TimedOut,
        )),
    };
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}

pub(in crate::domain_computation) fn principal_currentness(
    denied: crate::domain_computation::vocabulary::primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenial,
) -> Kind {
    use crate::domain_computation::vocabulary::primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenialKind as Principal;
    match denied.kind() {
        Principal::Stopped(stop) => Kind::Stopped(stop),
        Principal::Refused(_) => Kind::Refused(AuthorizationRefusal::StalePrincipal),
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
        Stop::AccountingUnavailable => Cause::AccountingUnavailable,
        Stop::NestedStopped => Cause::NestedStopped,
    };
    denial(cause, "authorization request").kind()
}
