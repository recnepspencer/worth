//! Synchronous authentication of the local operating-system process.
use std::time::{Duration, SystemTime};

use worth_query_declaration::facade::authentication::{
    WorthQueryAuthenticationAudienceIdentity, WorthQueryAuthenticationMethodIdentity,
    WorthQueryExternalPrincipalIdentity,
};

use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

use super::{
    admit_authentication_adapter, WorthQueryAdmissionClock,
    WorthQueryAdmittedAuthenticationAdapter, WorthQueryAuthenticatedExternalPrincipal,
    WorthQueryAuthenticationAdapter, WorthQueryAuthenticationAdapterAdmission,
    WorthQueryAuthenticationAdapterAdmissionDenial, WorthQueryAuthenticationAdapterFailure,
    WorthQueryAuthenticationAdapterFailureKind, WorthQueryAuthenticationAudience,
    WorthQueryAuthenticationDenial, WorthQueryAuthenticationDenialKind,
    WorthQueryAuthenticationFuture, WorthQueryAuthenticationMethod, WorthQueryRequestScope,
    WorthQueryValidatedExternalPrincipal, WorthQueryValidatedExternalPrincipalDenial,
};

/// Every local-process adapter shares one configuration; its audience and
/// method are fixed by the declared vocabulary at construction.
const LOCAL_PROCESS_CONFIGURATION_IDENTITY: &str = "worth.query.local-process.v1";

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
/// worth_query_declaration::worth_query_authentication_audience!(Board);
/// worth_query_declaration::worth_query_authentication_method!(LocalProcess);
/// worth_query_declaration::worth_query_principal_issuer!(BoardLocal);
///
/// let owner = || WorthQueryExternalPrincipalIdentity::new(BoardLocal, "owner").unwrap();
/// let local = WorthQueryLocalProcessPrincipal::new(
///     owner(), Board, LocalProcess, Duration::from_secs(600),
/// )
/// .unwrap();
/// assert_eq!(local.lifetime(), Duration::from_secs(600));
/// assert!(
///     WorthQueryLocalProcessPrincipal::new(owner(), Board, LocalProcess, Duration::ZERO).is_err()
/// );
/// ```
pub struct WorthQueryLocalProcessPrincipal {
    identity: WorthQueryExternalPrincipalIdentity,
    audience: WorthQueryAuthenticationAudience,
    method: WorthQueryAuthenticationMethod,
    lifetime: Duration,
}

impl WorthQueryLocalProcessPrincipal {
    /// The local process acting as `identity` for the declared `Audience`
    /// and `Method`. Refuses a zero lifetime: every local proof expires.
    pub fn new<Audience, Method>(
        identity: WorthQueryExternalPrincipalIdentity,
        audience: Audience,
        method: Method,
        lifetime: Duration,
    ) -> Result<Self, WorthQueryValidatedExternalPrincipalDenial>
    where
        Audience: WorthQueryAuthenticationAudienceIdentity,
        Method: WorthQueryAuthenticationMethodIdentity,
    {
        if lifetime.is_zero() {
            return Err(WorthQueryValidatedExternalPrincipalDenial::InvalidTimeRange);
        }
        Ok(Self {
            identity,
            audience: WorthQueryAuthenticationAudience::of(audience),
            method: WorthQueryAuthenticationMethod::of(method),
            lifetime,
        })
    }

    pub fn identity(&self) -> &WorthQueryExternalPrincipalIdentity {
        &self.identity
    }

    pub const fn lifetime(&self) -> Duration {
        self.lifetime
    }

    /// Admits this principal as its own adapter against `schema`, accepting
    /// exactly its own audience and method under `clock`.
    pub fn admit<Schema: ApplicationSchema>(
        self,
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
        clock: WorthQueryAdmissionClock,
    ) -> Result<
        WorthQueryAdmittedAuthenticationAdapter<Schema, Self>,
        WorthQueryAuthenticationAdapterAdmissionDenial,
    > {
        let admission = WorthQueryAuthenticationAdapterAdmission::new(
            self.audience.clone(),
            self.method.clone(),
            clock,
        );
        admit_authentication_adapter(schema, admission, self)
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
        LOCAL_PROCESS_CONFIGURATION_IDENTITY
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
            .field("identity", &self.identity)
            .field("audience", &self.audience)
            .field("method", &self.method)
            .field("lifetime", &self.lifetime)
            .finish()
    }
}
