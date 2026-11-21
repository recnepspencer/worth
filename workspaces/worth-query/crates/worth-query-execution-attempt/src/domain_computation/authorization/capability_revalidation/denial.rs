//! Refusal evidence for an active capability refresh.
pub use super::*;
pub fn inconsistent_refresh(
    refresh: &WorthQueryCapabilityRefresh<'_>,
) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::InconsistentDecision,
        ),
        refresh.installed.contract().name(),
    )
}

pub fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
