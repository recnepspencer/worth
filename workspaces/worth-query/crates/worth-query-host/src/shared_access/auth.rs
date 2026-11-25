//! Authenticated remote session credentials and single-pass WORTH principal admission.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_query_admission::facade::authenticated_principal::{
    admit_authentication_adapter, WorthQueryAdmissionClock,
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryAuthenticationAdapter,
    WorthQueryAuthenticationAdapterAdmission, WorthQueryAuthenticationAdapterFailure,
    WorthQueryAuthenticationAdapterFailureKind, WorthQueryAuthenticationAudience,
    WorthQueryAuthenticationFuture, WorthQueryAuthenticationMethod, WorthQueryRequestScope,
    WorthQueryValidatedExternalPrincipal,
};
use worth_query_declaration::facade::{
    application_schema::ApplicationSchema, authentication::WorthQueryExternalPrincipalIdentity,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use super::capped_writer::RemoteByteWriter;
use super::denial::{AuthenticationRefusalReason, SharedAccessDenial};
use super::framing::{BinaryReader, BinaryWriter};
use super::hmac::{
    compute_token_mac, constant_time_eq_32, decode_token_parts, encode_token_body,
    encode_token_body_capped, TokenBody, MAX_KEY_ID_BYTES, MAX_TOKEN_BYTES,
};
use super::types::RemoteServerLimits;

/// Named 256-bit HMAC signing key. Secret material is never rendered by `Debug`.
#[derive(Clone, Eq, PartialEq)]
pub struct RemoteAuthKey {
    key_id: String,
    secret: [u8; 32],
}

impl RemoteAuthKey {
    pub fn new(key_id: impl Into<String>, secret: [u8; 32]) -> Result<Self, SharedAccessDenial> {
        let key_id = key_id.into();
        if key_id.is_empty()
            || key_id.len() > MAX_KEY_ID_BYTES
            || key_id.trim() != key_id
            || key_id.chars().any(char::is_control)
            || secret == [0_u8; 32]
        {
            return Err(SharedAccessDenial::InvalidAuthConfiguration);
        }
        Ok(Self { key_id, secret })
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }
}

impl std::fmt::Debug for RemoteAuthKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoteAuthKey")
            .field("key_id", &self.key_id)
            .field("secret", &"[redacted]")
            .finish()
    }
}

/// Parameters for issuing one authenticated remote session token.
#[derive(Clone, Debug)]
pub struct RemoteTokenIssueRequest {
    pub key_id: String,
    pub home_identity: HomeIdentity,
    pub pinned_incarnation: Option<HomeIncarnation>,
    pub principal: WorthQueryExternalPrincipalIdentity,
    pub ttl: Duration,
    pub nonce: [u8; 16],
}

/// Server-side authentication configuration, key set, and bounded budgets.
#[derive(Clone, Debug)]
pub struct RemoteAuthConfig {
    audience: WorthQueryAuthenticationAudience,
    method: WorthQueryAuthenticationMethod,
    keys: BTreeMap<String, RemoteAuthKey>,
    limits: RemoteServerLimits,
    require_incarnation_pin: bool,
}

impl RemoteAuthConfig {
    pub fn new(
        audience: WorthQueryAuthenticationAudience,
        method: WorthQueryAuthenticationMethod,
        initial_key: RemoteAuthKey,
    ) -> Self {
        let mut keys = BTreeMap::new();
        keys.insert(initial_key.key_id.clone(), initial_key);
        Self {
            audience,
            method,
            keys,
            limits: RemoteServerLimits::default(),
            require_incarnation_pin: false,
        }
    }

    pub fn with_key(mut self, key: RemoteAuthKey) -> Self {
        self.keys.insert(key.key_id.clone(), key);
        self
    }

    pub fn with_limits(mut self, limits: RemoteServerLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Requires every presented token to carry a `pinned_incarnation` matching the
    /// live Directory Home incarnation. When `false` (default), earlier pinned tokens
    /// still refuse with `StaleIncarnation` after reopen, while valid unpinned tokens
    /// bound to the same `HomeIdentity` continue to authenticate.
    pub const fn with_require_incarnation_pin(mut self, require: bool) -> Self {
        self.require_incarnation_pin = require;
        self
    }

    pub const fn limits(&self) -> RemoteServerLimits {
        self.limits
    }

    pub const fn require_incarnation_pin(&self) -> bool {
        self.require_incarnation_pin
    }

    pub fn audience(&self) -> &WorthQueryAuthenticationAudience {
        &self.audience
    }

    pub fn method(&self) -> &WorthQueryAuthenticationMethod {
        &self.method
    }

    /// Issue a signed HMAC-SHA256 bearer token bound to `home_identity` and `principal`.
    pub fn issue_token(
        &self,
        request: RemoteTokenIssueRequest,
    ) -> Result<RemoteAuthToken, SharedAccessDenial> {
        if request.ttl.is_zero() {
            return Err(SharedAccessDenial::InvalidAuthConfiguration);
        }
        let key =
            self.keys
                .get(&request.key_id)
                .ok_or(SharedAccessDenial::AuthenticationRefused(
                    AuthenticationRefusalReason::UnknownKey,
                ))?;
        let now_secs = current_unix_secs()?;
        let expires_secs = now_secs
            .checked_add(request.ttl.as_secs().max(1))
            .ok_or(SharedAccessDenial::InvalidAuthConfiguration)?;
        let body = TokenBody {
            key_id: request.key_id,
            home_identity: request.home_identity.bytes(),
            pinned_incarnation: request.pinned_incarnation.map(HomeIncarnation::get),
            principal: request.principal,
            issued_at_unix_secs: now_secs,
            expires_at_unix_secs: expires_secs,
            nonce: request.nonce,
        };
        let mac = compute_token_mac(&key.secret, &body);
        Ok(RemoteAuthToken { body, mac })
    }
}

/// Authenticated HMAC-SHA256 bearer token presented by a remote collaborator.
#[derive(Clone, Eq, PartialEq)]
pub struct RemoteAuthToken {
    body: TokenBody,
    mac: [u8; 32],
}

impl RemoteAuthToken {
    pub fn key_id(&self) -> &str {
        &self.body.key_id
    }

    pub fn principal(&self) -> &WorthQueryExternalPrincipalIdentity {
        &self.body.principal
    }

    pub const fn pinned_incarnation(&self) -> Option<u64> {
        self.body.pinned_incarnation
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = BinaryWriter::with_capacity(256);
        encode_token_body(&mut writer, &self.body);
        writer.write_fixed(&self.mac);
        writer.into_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SharedAccessDenial> {
        if bytes.len() > MAX_TOKEN_BYTES {
            return Err(SharedAccessDenial::AuthenticationRefused(
                AuthenticationRefusalReason::MalformedToken,
            ));
        }
        let mut reader = BinaryReader::new(bytes);
        let (body, mac) = decode_token_parts(&mut reader).map_err(|_| {
            SharedAccessDenial::AuthenticationRefused(AuthenticationRefusalReason::MalformedToken)
        })?;
        if reader.remaining() != 0 {
            return Err(SharedAccessDenial::AuthenticationRefused(
                AuthenticationRefusalReason::MalformedToken,
            ));
        }
        Ok(Self { body, mac })
    }

    pub(super) fn encode_into_capped(
        &self,
        writer: &mut RemoteByteWriter,
    ) -> Result<(), SharedAccessDenial> {
        writer.write_length_prefixed(|inner| {
            encode_token_body_capped(inner, &self.body)?;
            inner.write_fixed(&self.mac)
        })
    }

    pub(super) fn decode_from(reader: &mut BinaryReader<'_>) -> Result<Self, SharedAccessDenial> {
        let slice = reader.read_bytes_bounded(MAX_TOKEN_BYTES)?;
        Self::from_bytes(slice)
    }
}

impl std::fmt::Debug for RemoteAuthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoteAuthToken")
            .field("key_id", &self.body.key_id)
            .field("pinned_incarnation", &self.body.pinned_incarnation)
            .field("expires_at_unix_secs", &self.body.expires_at_unix_secs)
            .field("mac", &"[redacted]")
            .finish()
    }
}

/// Single-use WORTH admission adapter carrying a pre-verified principal with zero key cloning.
struct PreValidatedTokenAdapter {
    validated: Mutex<Option<WorthQueryValidatedExternalPrincipal>>,
}

impl WorthQueryAuthenticationAdapter for PreValidatedTokenAdapter {
    type Credential = ();

    fn configuration_identity(&self) -> &str {
        "worth.query.host.shared_access.hmac_sha256.v1"
    }

    fn validate<'a>(
        &'a self,
        _credential: Self::Credential,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        let result = self
            .validated
            .lock()
            .ok()
            .and_then(|mut guard| guard.take())
            .ok_or_else(|| {
                WorthQueryAuthenticationAdapterFailure::new(
                    WorthQueryAuthenticationAdapterFailureKind::CredentialRejected,
                )
            });
        Box::pin(std::future::ready(result))
    }
}

pub(super) struct RemoteAuthTarget<'a> {
    pub(super) config: &'a RemoteAuthConfig,
    pub(super) home_identity: HomeIdentity,
    pub(super) incarnation: HomeIncarnation,
}

pub(super) struct AuthenticatedRemotePrincipal<Schema: ApplicationSchema> {
    pub(super) external: WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) expires_at_unix_secs: u64,
}

/// Verifies `token` strictly once by borrowing key material without cloning,
/// then mints a genuine WORTH external principal proof.
pub(super) fn authenticate_remote_token<Schema: ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    target: &RemoteAuthTarget<'_>,
    token: &RemoteAuthToken,
    request_scope: &WorthQueryRequestScope,
) -> Result<AuthenticatedRemotePrincipal<Schema>, SharedAccessDenial> {
    let (validated, expires_at_unix_secs) = verify_token_single_pass(target, token)
        .map_err(SharedAccessDenial::AuthenticationRefused)?;
    let adapter = PreValidatedTokenAdapter {
        validated: Mutex::new(Some(validated)),
    };
    let admitted = admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            target.config.audience.clone(),
            target.config.method.clone(),
            WorthQueryAdmissionClock::system(),
        ),
        adapter,
    )
    .map_err(|_| {
        SharedAccessDenial::AuthenticationRefused(AuthenticationRefusalReason::AdapterDenied)
    })?;
    let external = poll_ready_once(admitted.authenticate((), request_scope)).map_err(|_| {
        SharedAccessDenial::AuthenticationRefused(AuthenticationRefusalReason::AdapterDenied)
    })?;
    Ok(AuthenticatedRemotePrincipal {
        external,
        expires_at_unix_secs,
    })
}

fn verify_token_single_pass(
    target: &RemoteAuthTarget<'_>,
    token: &RemoteAuthToken,
) -> Result<(WorthQueryValidatedExternalPrincipal, u64), AuthenticationRefusalReason> {
    let key = target
        .config
        .keys
        .get(&token.body.key_id)
        .ok_or(AuthenticationRefusalReason::UnknownKey)?;
    let expected_mac = compute_token_mac(&key.secret, &token.body);
    if !constant_time_eq_32(&expected_mac, &token.mac) {
        return Err(AuthenticationRefusalReason::InvalidSignature);
    }
    if token.body.home_identity != target.home_identity.bytes() {
        return Err(AuthenticationRefusalReason::HomeIdentityMismatch);
    }
    match token.body.pinned_incarnation {
        Some(pin) if pin != target.incarnation.get() => {
            return Err(AuthenticationRefusalReason::StaleIncarnation);
        }
        None if target.config.require_incarnation_pin => {
            return Err(AuthenticationRefusalReason::IncarnationPinRequired);
        }
        _ => {}
    }
    let now_secs = current_unix_secs().map_err(|_| AuthenticationRefusalReason::TokenExpired)?;
    if token.body.issued_at_unix_secs > now_secs || token.body.expires_at_unix_secs <= now_secs {
        return Err(AuthenticationRefusalReason::TokenExpired);
    }
    let validated_at = UNIX_EPOCH + Duration::from_secs(token.body.issued_at_unix_secs);
    let expires_at = UNIX_EPOCH + Duration::from_secs(token.body.expires_at_unix_secs);
    let validated = WorthQueryValidatedExternalPrincipal::new(
        token.body.principal.clone(),
        target.config.audience.clone(),
        target.config.method.clone(),
        validated_at,
        expires_at,
        Vec::new(),
    )
    .map_err(|_| AuthenticationRefusalReason::MalformedToken)?;
    Ok((validated, token.body.expires_at_unix_secs))
}

fn poll_ready_once<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    match Pin::as_mut(&mut future).poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => unreachable!("PreValidatedTokenAdapter resolves immediately"),
    }
}

pub(super) fn current_unix_secs() -> Result<u64, SharedAccessDenial> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| SharedAccessDenial::HostUnavailable)
}
