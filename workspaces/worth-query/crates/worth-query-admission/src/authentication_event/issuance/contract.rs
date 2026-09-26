use std::future::Future;
use std::pin::Pin;

use worth_query_installation::facade::WorthQueryNamedClockFailureKind;

use crate::authenticated_principal::WorthQueryRequestScope;
use crate::authentication_event::clock::AuthenticationEventClockDenial;
use crate::authentication_event::intent::WorthQueryAuthenticationEventChallenge;

pub type WorthQueryAuthenticationEventFuture<'a> = Pin<
    Box<dyn Future<Output = Result<(), WorthQueryAuthenticationEventVerifierFailure>> + Send + 'a>,
>;

/// Installed host authentication checks the owner-issued challenge against a
/// real credential. A principal proof or caller boolean alone cannot issue it.
pub trait WorthQueryAuthenticationEventVerifier: Send + Sync + 'static {
    type Credential: Send + 'static;

    fn configuration_identity(&self) -> &str;

    fn verify<'a>(
        &'a self,
        credential: Self::Credential,
        challenge: &'a WorthQueryAuthenticationEventChallenge,
        scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationEventFuture<'a>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAuthenticationEventVerifierFailure {
    CredentialRejected,
    CredentialExpired,
    CredentialRevoked,
    DependencyUnavailable,
    ProtocolViolation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAuthenticationEventDenial {
    InvalidInstallation,
    WrongSchemaBinding,
    WrongOwner,
    PrincipalMismatch,
    IntentMismatch,
    PrincipalExpired,
    EventExpired,
    EventRevoked,
    EventUnavailable,
    MissingSigningProof,
    CapacityExceeded,
    EntropyUnavailable,
    IdentityExhausted,
    ClockRangeExceeded,
    ClockUnavailable(WorthQueryNamedClockFailureKind),
    ClockIdentityChanged,
    ClockRegressed,
    VerifierFailed(WorthQueryAuthenticationEventVerifierFailure),
    Cancelled,
    DeadlineExceeded,
}

impl From<AuthenticationEventClockDenial> for WorthQueryAuthenticationEventDenial {
    fn from(value: AuthenticationEventClockDenial) -> Self {
        match value {
            AuthenticationEventClockDenial::InvalidInstallation => Self::InvalidInstallation,
            AuthenticationEventClockDenial::Source(kind) => Self::ClockUnavailable(kind),
            AuthenticationEventClockDenial::IdentityChanged => Self::ClockIdentityChanged,
            AuthenticationEventClockDenial::Regressed => Self::ClockRegressed,
        }
    }
}
