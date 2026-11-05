//! The retained request authority exposes execution causes without changing ordinary authorization.

pub(super) fn authority_execution(
    kind: crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind,
) -> Option<
    Result<
        crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind,
    >,
> {
    match kind {
        crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind::Stopped(stop) => Some(stop),
        crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind::Refused(_) => None,
    }
}
