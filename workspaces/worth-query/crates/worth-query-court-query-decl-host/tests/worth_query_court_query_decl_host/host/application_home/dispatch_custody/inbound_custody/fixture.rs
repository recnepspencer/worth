//! The real Home and a trusted exact-body verifier; signed rail evidence belongs to Bank.
use super::*;
use dispatch::{inbound_action::InboundNoticeIntent, inbound_source as sources};
use std::collections::BTreeMap;
use std::sync::atomic::AtomicU64;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_host::facade::primary_graph::{
    WorthQueryInboundOccurrenceClaims, WorthQueryInboundOccurrenceVerifier,
    WorthQueryInboundVerificationDenial, WorthQueryInboundVerifierHandle,
    WorthQueryRuntimeTimeSource, WorthQueryRuntimeTimeSourceDenial,
};

pub(super) struct Owners {
    pub location: std::path::PathBuf,
    pub handlers: Arc<AtomicUsize>,
    pub now: Arc<AtomicU64>,
}
struct Clock(Arc<AtomicU64>);
impl WorthQueryRuntimeTimeSource for Clock {
    fn admission_clock(
        &self,
    ) -> worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock
    {
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()
    }
    fn current_time(&self) -> Result<std::time::SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        Ok(std::time::UNIX_EPOCH + Duration::from_secs(self.0.load(Ordering::SeqCst)))
    }
}
impl Owners {
    pub fn new() -> Self {
        Self {
            location: dispatch::location(),
            handlers: Arc::new(AtomicUsize::new(0)),
            now: Arc::new(AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )),
        }
    }
    pub fn open(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        seed: bool,
        outcomes: &[Outcome],
    ) -> (Arc<Home>, Arc<ScriptedRail>) {
        let request = dispatch::request(
            self.location.clone(),
            ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
            self.handlers.clone(),
            seed,
        )
        .with_authorization_time_source(Clock(self.now.clone()));
        let home = with_bootstrap_advancement_from_request(execution, |phase| {
            request.open_in_advancement(&phase)
        })
        .unwrap()
        .unwrap_or_else(|refused| {
            super::super::super::reopening::opening_failed("inbound Directory Home", &refused)
        });
        let home = Arc::new(home);
        let rail = ScriptedRail::answering(outcomes);
        home.runtime()
            .runtime()
            .install_external_effect_transport(rail.clone())
            .unwrap();
        rail.home
            .set(Arc::downgrade(&home))
            .unwrap_or_else(|_| panic!("one rail serves one opening"));
        (home, rail)
    }
    pub fn expire_messages(&self) {
        self.now.fetch_add(31, Ordering::SeqCst);
    }
}

pub(super) struct Effect {
    pub token: [u8; 32],
    pub correlation: worth_query_host::facade::primary_graph::ExternalEffectCorrelationIdentity,
    payload: Vec<u8>,
    protocol: BoundaryProtocolIdentity,
    version: BoundaryProtocolVersion,
    family: String,
    source: String,
}
pub(super) fn emit(home: &Home, key: u64, scope: &WorthQueryRequestScope) -> Effect {
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        scope,
    );
    let outcome = runtime
        .request(&principal, scope)
        .advanced()
        .mutate(InboundNoticeIntent(key))
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
        panic!("real inbound-capable emission: {outcome:?}")
    };
    assert_eq!(result, key);
    let record = receipt
        .dispatch_outbox()
        .expect("a genuine co-committed outbox");
    let effect = Effect {
        token: *record.correlation().bytes(),
        correlation: *record.correlation(),
        payload: record.payload().to_vec(),
        protocol: record.protocol_identity().clone(),
        version: record.protocol_version(),
        family: record.correlation_family().as_str().to_owned(),
        source: record.inbound().unwrap().source_identity().to_owned(),
    };
    assert!(
        receipt.external_dispatch().is_some(),
        "the actual transport was contacted"
    );
    effect
}

pub(super) struct Route {
    pub handle: WorthQueryInboundVerifierHandle,
    verifier: Arc<PinnedVerifier>,
}
impl Route {
    pub fn install(home: &Home, audience: &str) -> Self {
        let verifier = Arc::new(PinnedVerifier {
            audience: audience.to_owned(),
            messages: Mutex::new(BTreeMap::new()),
            protocol: sources::protocol_identity(),
        });
        let operation = home
            .runtime()
            .runtime()
            .installed_schema()
            .installed_operation(sources::InboundNotice::reference())
            .unwrap();
        let handle = home
            .runtime()
            .runtime()
            .install_inbound_occurrence_verifier(&operation, verifier.clone())
            .unwrap();
        Self { handle, verifier }
    }
    pub fn message(&self, owners: &Owners, effect: &Effect, nonce: u8) -> Vec<u8> {
        self.message_at(owners, effect, nonce, 30)
    }
    pub fn message_at(
        &self,
        owners: &Owners,
        effect: &Effect,
        nonce: u8,
        validity: u64,
    ) -> Vec<u8> {
        let now = owners.now.load(Ordering::SeqCst);
        let mut claims = WorthQueryInboundOccurrenceClaims {
            audience: self.verifier.audience.clone(),
            source_identity: effect.source.clone(),
            key_epoch: 1,
            message_identity: [nonce; 32],
            signed_meaning_digest: [0; 32],
            issued_at_unix_seconds: now,
            expires_at_unix_seconds: now + validity,
            protocol_identity: effect.protocol.clone(),
            protocol_version: effect.version,
            correlation_family: effect.family.clone(),
            correlation_token: effect.token,
            payload: effect.payload.clone(),
        };
        let body = serde_json::json!([
            claims.audience,
            claims.source_identity,
            claims.key_epoch,
            claims.message_identity.to_vec(),
            claims.issued_at_unix_seconds,
            claims.expires_at_unix_seconds,
            claims.protocol_identity.as_str(),
            claims.protocol_version.get(),
            claims.correlation_family,
            claims.correlation_token.to_vec(),
            claims.payload
        ])
        .to_string();
        claims.signed_meaning_digest = super::super::source_controls::signed_meaning(&body);
        self.verifier
            .messages
            .lock()
            .unwrap()
            .insert(body.as_bytes().to_vec(), claims);
        body.into_bytes()
    }
}
struct PinnedVerifier {
    audience: String,
    protocol: BoundaryProtocolIdentity,
    messages: Mutex<BTreeMap<Vec<u8>, WorthQueryInboundOccurrenceClaims>>,
}
impl WorthQueryInboundOccurrenceVerifier for PinnedVerifier {
    fn audience(&self) -> &str {
        &self.audience
    }
    fn source_identity(&self) -> &str {
        sources::SOURCE
    }
    fn protocol_identity(&self) -> &BoundaryProtocolIdentity {
        &self.protocol
    }
    fn protocol_version(&self) -> BoundaryProtocolVersion {
        BoundaryProtocolVersion::new(1)
    }
    fn verify(
        &self,
        envelope: &[u8],
        now: u64,
        work: std::num::NonZeroU64,
    ) -> Result<WorthQueryInboundOccurrenceClaims, WorthQueryInboundVerificationDenial> {
        if work.get() < (envelope.len() as u64).saturating_mul(2) {
            return Err(WorthQueryInboundVerificationDenial::WorkExhausted);
        }
        let messages = self.messages.lock().unwrap();
        let claims = messages
            .get(envelope)
            .ok_or(WorthQueryInboundVerificationDenial::AuthenticationFailed)?;
        if now < claims.issued_at_unix_seconds || now > claims.expires_at_unix_seconds {
            return Err(WorthQueryInboundVerificationDenial::Expired);
        }
        Ok(claims.clone())
    }
}
pub(super) fn receive(home: &Home, route: &Route, body: &[u8]) -> Result<Posture, Denial> {
    let (_, scope) = pending_support::scope();
    home.runtime()
        .runtime()
        .receive_inbound_occurrence(&route.handle, body, &scope)
        .map(|receipt| {
            assert_ne!(
                receipt.posture(),
                Posture::AcceptedPending,
                "terminal callback remains pending: {:?}; Native preparation: {:?}",
                receipt.pending_reason(),
                receipt.preparation_denial()
            );
            receipt.posture()
        })
}
pub(super) fn close(home: Arc<Home>, execution: &worth_execution::ExecutionRequest<'_, '_>) {
    Arc::into_inner(home)
        .expect("no live Home alias")
        .close(execution)
        .unwrap_or_else(|refused| {
            panic!("inbound custody has a lawful durable handoff: {refused:?}")
        })
        .reclaim();
}
pub(super) fn terminal(home: &Home, route: &Route, effect: &Effect) -> (u64, u64, u64) {
    let terminal = home
        .runtime()
        .runtime()
        .observe_inbound_terminal(&route.handle, effect.token)
        .expect("canonical original completion");
    (
        terminal.original_world_commit().ordinal(),
        terminal.completion_world_commit().ordinal(),
        terminal.completion_attempt().ordinal(),
    )
}
pub(super) fn maintain(
    home: &Home,
    route: &Route,
) -> worth_query_host::facade::primary_graph::WorthQueryInboundMaintenanceReport {
    let (_, scope) = pending_support::scope();
    home.runtime()
        .runtime()
        .maintain_inbound_occurrences(&route.handle, NonZeroUsize::new(16).unwrap(), &scope)
        .unwrap()
}
pub(super) fn assert_conflict(result: Result<Posture, Denial>, expected: Conflict) {
    let Err(Denial::AuthenticatedPermanent(actual)) = result else {
        panic!("authenticated permanent conflict: {result:?}")
    };
    assert_eq!(actual.kind(), expected);
}

// D5 and Shared durable owner contracts, Live abandonment and recovered
// disposition: live durable custody requires ending this Home and reopening.
// Close refuses it; only actual cold reconciliation grants continuation.
pub(super) fn end_live_home_for_required_reopen(
    home: Arc<Home>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let refused = Arc::into_inner(home)
        .expect("no live Home alias")
        .close(execution)
        .err()
        .expect("live original custody requires Home reopen");
    assert!(refused.directory_denial().is_some());
    assert!(refused.query_denial().is_none());
    let serving = refused
        .resume()
        .expect("refused close retains the same serving Home");
    drop(serving);
}
