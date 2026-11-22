//! Synchronous authentication of the local operating-system process.
use std::time::{Duration, SystemTime};

use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;

use super::{
    WorthQueryAdmittedAuthenticationAdapter, WorthQueryAuthenticatedExternalPrincipal,
    WorthQueryAuthenticationAdapter, WorthQueryAuthenticationAdapterFailure,
    WorthQueryAuthenticationAdapterFailureKind, WorthQueryAuthenticationAudience,
    WorthQueryAuthenticationDenial, WorthQueryAuthenticationDenialKind,
    WorthQueryAuthenticationFuture, WorthQueryAuthenticationMethod, WorthQueryRequestScope,
    WorthQueryValidatedExternalPrincipal, WorthQueryValidatedExternalPrincipalDenial,
};

/// The external identity a desktop or single-host process acts as, with a
/// finite credential lifetime.
///
/// It is an ordinary authentication adapter: admit it with
/// [`admit_authentication_adapter`](super::admit_authentication_adapter), which
/// fixes the accepted audience, method, clock and adapter identity exactly as
/// for a remote adapter. [`WorthQueryAdmittedAuthenticationAdapter::authenticate_local`]
/// then mints the same sealed proof without an executor. Installed principal
/// mapping, disabled mappings and proof expiry are enforced by request
/// admission, identically to remote credentials.
///
/// ```
/// use std::time::Duration;
/// use worth_query_admission::facade::authenticated_principal::*;
/// use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
///
/// let local = WorthQueryLocalProcessPrincipal::new(
///     "board.local-owner.v1",
///     WorthQueryExternalPrincipalIdentity::new("local://board", "owner").unwrap(),
///     WorthQueryAuthenticationAudience::new("board").unwrap(),
///     WorthQueryAuthenticationMethod::new("local-process").unwrap(),
///     Duration::from_secs(600),
/// )
/// .unwrap();
/// assert_eq!(local.lifetime(), Duration::from_secs(600));
/// assert!(WorthQueryLocalProcessPrincipal::new(
///     "board.local-owner.v1",
///     WorthQueryExternalPrincipalIdentity::new("local://board", "owner").unwrap(),
///     WorthQueryAuthenticationAudience::new("board").unwrap(),
///     WorthQueryAuthenticationMethod::new("local-process").unwrap(),
///     Duration::ZERO,
/// )
/// .is_err());
/// ```
pub struct WorthQueryLocalProcessPrincipal {
    configuration_identity: String,
    identity: WorthQueryExternalPrincipalIdentity,
    audience: WorthQueryAuthenticationAudience,
    method: WorthQueryAuthenticationMethod,
    lifetime: Duration,
}

impl WorthQueryLocalProcessPrincipal {
    /// Refuses a zero lifetime: every local proof expires.
    pub fn new(
        configuration_identity: impl Into<String>,
        identity: WorthQueryExternalPrincipalIdentity,
        audience: WorthQueryAuthenticationAudience,
        method: WorthQueryAuthenticationMethod,
        lifetime: Duration,
    ) -> Result<Self, WorthQueryValidatedExternalPrincipalDenial> {
        if lifetime.is_zero() {
            return Err(WorthQueryValidatedExternalPrincipalDenial::InvalidTimeRange);
        }
        Ok(Self {
            configuration_identity: configuration_identity.into(),
            identity,
            audience,
            method,
            lifetime,
        })
    }

    pub fn identity(&self) -> &WorthQueryExternalPrincipalIdentity {
        &self.identity
    }

    pub const fn lifetime(&self) -> Duration {
        self.lifetime
    }

    /// The local process vouches for itself now, for the configured lifetime.
    fn candidate(
        &self,
    ) -> Result<WorthQueryValidatedExternalPrincipal, WorthQueryAuthenticationAdapterFailure> {
        let protocol_violation = || {
            WorthQueryAuthenticationAdapterFailure::new(
                WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
            )
        };
        let validated_at = SystemTime::now();
        let expires_at = validated_at
            .checked_add(self.lifetime)
            .ok_or_else(protocol_violation)?;
        WorthQueryValidatedExternalPrincipal::new(
            self.identity.clone(),
            self.audience.clone(),
            self.method.clone(),
            validated_at,
            expires_at,
            Vec::new(),
        )
        .map_err(|_| protocol_violation())
    }
}

impl WorthQueryAuthenticationAdapter for WorthQueryLocalProcessPrincipal {
    type Credential = ();

    fn configuration_identity(&self) -> &str {
        &self.configuration_identity
    }

    fn validate<'a>(
        &'a self,
        _credential: (),
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(std::future::ready(self.candidate()))
    }
}

impl<Schema> WorthQueryAdmittedAuthenticationAdapter<Schema, WorthQueryLocalProcessPrincipal> {
    /// Authenticates the local process on the calling thread. The scope,
    /// audience, method, validation time and expiry checks are the ones
    /// [`authenticate`](Self::authenticate) applies to every adapter.
    pub fn authenticate_local(
        &self,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryAuthenticatedExternalPrincipal<Schema>, WorthQueryAuthenticationDenial>
    {
        self.admit_scope(scope)?;
        let candidate = self.adapter().candidate().map_err(|failure| {
            WorthQueryAuthenticationDenial::new(WorthQueryAuthenticationDenialKind::AdapterFailed(
                failure.kind(),
            ))
        })?;
        self.admit_candidate(candidate)
    }
}

impl std::fmt::Debug for WorthQueryLocalProcessPrincipal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryLocalProcessPrincipal")
            .field("configuration_identity", &self.configuration_identity)
            .field("identity", &self.identity)
            .field("audience", &self.audience)
            .field("method", &self.method)
            .field("lifetime", &self.lifetime)
            .finish()
    }
}
