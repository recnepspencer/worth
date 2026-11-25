use std::time::{Duration, Instant};

use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticationAudience, WorthQueryAuthenticationMethod, WorthQueryCancellationSource,
    WorthQueryLocalProcessPrincipal, WorthQueryRequestScope,
};
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;

use crate::BankAuthenticationConfiguration;

worth_query_decl::facade::worth_query_authentication_audience!(BankPhase7CapabilityTestAudience);
worth_query_decl::facade::worth_query_authentication_method!(CausalPhase7AdapterMethod);
worth_query_decl::facade::worth_query_principal_issuer!(BankPhase7TestIssuer);

pub(super) fn external_identity(
    scenario: &str,
    subject: &str,
) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(BankPhase7TestIssuer, format!("{scenario}:{subject}"))
        .unwrap()
}

pub(crate) fn request_scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        WorthQueryCancellationSource::new().token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
}

pub(super) fn authentication_configuration() -> BankAuthenticationConfiguration {
    BankAuthenticationConfiguration::new(
        WorthQueryAuthenticationAudience::of(BankPhase7CapabilityTestAudience),
        WorthQueryAuthenticationMethod::of(CausalPhase7AdapterMethod),
    )
}

/// The Host's local process principal acting as one mapped test identity.
pub(super) fn local_principal(
    identity: WorthQueryExternalPrincipalIdentity,
) -> WorthQueryLocalProcessPrincipal {
    WorthQueryLocalProcessPrincipal::new(
        identity,
        BankPhase7CapabilityTestAudience,
        CausalPhase7AdapterMethod,
        Duration::from_secs(60),
    )
    .unwrap()
}
