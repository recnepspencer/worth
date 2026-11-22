use std::time::{Duration, Instant};

use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticationAudience, WorthQueryAuthenticationMethod, WorthQueryCancellationSource,
    WorthQueryLocalProcessPrincipal, WorthQueryRequestScope,
};
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;

use crate::BankAuthenticationConfiguration;

pub(super) fn external_identity(
    scenario: &str,
    subject: &str,
) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(
        format!("https://{scenario}.bank.test.invalid"),
        subject,
    )
    .unwrap()
}

pub(crate) fn request_scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        WorthQueryCancellationSource::new().token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
}

pub(super) fn authentication_configuration() -> BankAuthenticationConfiguration {
    BankAuthenticationConfiguration::new(
        WorthQueryAuthenticationAudience::new("bank-phase-7-capability-test").unwrap(),
        WorthQueryAuthenticationMethod::new("causal-phase-7-adapter").unwrap(),
    )
}

/// The Host's local process principal acting as one mapped test identity.
pub(super) fn local_principal(
    identity: WorthQueryExternalPrincipalIdentity,
) -> WorthQueryLocalProcessPrincipal {
    WorthQueryLocalProcessPrincipal::new(
        "bank-phase-7-capability-local-principal-v1",
        identity,
        WorthQueryAuthenticationAudience::new("bank-phase-7-capability-test").unwrap(),
        WorthQueryAuthenticationMethod::new("causal-phase-7-adapter").unwrap(),
        Duration::from_secs(60),
    )
    .unwrap()
}
