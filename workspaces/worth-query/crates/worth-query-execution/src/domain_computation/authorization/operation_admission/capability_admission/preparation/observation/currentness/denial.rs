//! Named currentness denials shared by the observation steps.

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

pub(super) fn stale_principal(
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    authorization_denial(
        WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::StalePrincipal),
        subject,
    )
}

pub(super) fn projection_rejected(
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    authorization_denial(
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityProjectionRejected,
        ),
        subject,
    )
}

pub(super) fn authorization_denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
