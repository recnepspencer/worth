//! Refusal evidence for an active capability refresh.
use super::*;
pub(super) fn inconsistent_refresh(
    refresh: &WorthQueryCapabilityRefresh<'_>,
) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::InconsistentDecision,
        ),
        refresh.installed.contract().name(),
    )
}

pub(super) fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
