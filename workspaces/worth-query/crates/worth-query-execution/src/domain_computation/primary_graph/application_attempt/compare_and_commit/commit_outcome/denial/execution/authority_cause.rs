//! The retained request authority exposes execution causes without changing ordinary authorization.

pub(super) fn authority_execution(
    kind: crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind,
) -> Option<
    Result<
        crate::domain_computation::WorthQueryProviderSessionDenialKind,
        crate::domain_computation::WorthQueryProviderSessionControlStopKind,
    >,
> {
    match kind {
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Stopped(stop) => Some(stop),
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(_) => None,
    }
}
