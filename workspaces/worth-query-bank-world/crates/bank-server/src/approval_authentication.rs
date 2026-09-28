//! Installed factor and named clock for Bank workflow approval signatures.

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bank_domain::schema::BankSchema;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::admission::authentication_event::{
    install_authentication_event_owner, WorthQueryAuthenticationEventChallenge,
    WorthQueryAuthenticationEventDenial, WorthQueryAuthenticationEventFuture,
    WorthQueryAuthenticationEventPolicy, WorthQueryAuthenticationEventReuse,
    WorthQueryAuthenticationEventVerifier, WorthQueryAuthenticationEventVerifierFailure,
    WorthQueryInstalledAuthenticationEventOwner,
};
use worth_query_host::facade::domain::{
    WorthQueryClockCoordinate, WorthQueryClockSourceIdentity, WorthQueryClockTimelineIdentity,
    WorthQueryInstalledApplicationSchema, WorthQueryNamedClock, WorthQueryNamedClockFailure,
    WorthQueryNamedClockFailureKind, WorthQueryNamedClockReading, WorthQueryNamedClockSource,
};

/// Opaque input for the installed factor. Bank never interprets or records it.
pub struct BankApprovalCredential(Vec<u8>);

impl BankApprovalCredential {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Debug for BankApprovalCredential {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("BankApprovalCredential([REDACTED])")
    }
}

/// A host supplied factor verifies the owner-issued challenge using the credential.
/// The default installation denies every approval until one is supplied.
pub struct BankApprovalAuthenticationConfiguration {
    verifier: Arc<dyn WorthQueryAuthenticationEventVerifier<Credential = BankApprovalCredential>>,
}

impl BankApprovalAuthenticationConfiguration {
    pub fn new(
        verifier: impl WorthQueryAuthenticationEventVerifier<Credential = BankApprovalCredential>,
    ) -> Self {
        Self {
            verifier: Arc::new(verifier),
        }
    }

    pub(crate) fn denying() -> Self {
        Self::new(DenyApprovalVerifier)
    }
}

struct DenyApprovalVerifier;

impl WorthQueryAuthenticationEventVerifier for DenyApprovalVerifier {
    type Credential = BankApprovalCredential;

    fn configuration_identity(&self) -> &str {
        "bank.workflow-approval.unconfigured.v1"
    }

    fn verify<'a>(
        &'a self,
        _credential: Self::Credential,
        _challenge: &'a WorthQueryAuthenticationEventChallenge,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationEventFuture<'a> {
        Box::pin(async { Err(WorthQueryAuthenticationEventVerifierFailure::CredentialRejected) })
    }
}

pub(crate) struct BankApprovalVerifier(
    Arc<dyn WorthQueryAuthenticationEventVerifier<Credential = BankApprovalCredential>>,
);

impl WorthQueryAuthenticationEventVerifier for BankApprovalVerifier {
    type Credential = BankApprovalCredential;

    fn configuration_identity(&self) -> &str {
        self.0.configuration_identity()
    }

    fn verify<'a>(
        &'a self,
        credential: Self::Credential,
        challenge: &'a WorthQueryAuthenticationEventChallenge,
        scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationEventFuture<'a> {
        self.0.verify(credential, challenge, scope)
    }
}

pub(crate) struct BankApprovalClock;

impl WorthQueryNamedClock for BankApprovalClock {
    const PORTABLE_IDENTITY: &'static str = "bank.workflow-approval.monotonic.v1";
}

pub(crate) struct BankApprovalClockSource {
    started: Instant,
    timeline: WorthQueryClockTimelineIdentity,
    sequence: AtomicU64,
}

impl BankApprovalClockSource {
    fn new() -> Result<Self, WorthQueryAuthenticationEventDenial> {
        let mut identity = [0_u8; 16];
        getrandom::fill(&mut identity)
            .map_err(|_| WorthQueryAuthenticationEventDenial::EntropyUnavailable)?;
        Ok(Self {
            started: Instant::now(),
            timeline: WorthQueryClockTimelineIdentity::declare(format!(
                "bank-workflow-approval-{:032x}",
                u128::from_be_bytes(identity)
            ))
            .expect("the generated timeline identity is valid"),
            sequence: AtomicU64::new(1),
        })
    }
}

impl WorthQueryNamedClockSource<BankApprovalClock> for BankApprovalClockSource {
    const SEMANTIC_IDENTITY: &'static str = "bank.workflow-approval.clock-source.v1";

    fn source_identity(&self) -> WorthQueryClockSourceIdentity {
        WorthQueryClockSourceIdentity::declare("bank-workflow-approval-monotonic")
            .expect("the installed source identity is valid")
    }

    fn timeline_identity(&self) -> WorthQueryClockTimelineIdentity {
        self.timeline.clone()
    }

    fn observe(
        &self,
    ) -> Result<WorthQueryNamedClockReading<BankApprovalClock>, WorthQueryNamedClockFailure> {
        let sequence = self
            .sequence
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                value.checked_add(1)
            })
            .map_err(|_| clock_range_exceeded())?;
        let elapsed =
            u64::try_from(self.started.elapsed().as_nanos()).map_err(|_| clock_range_exceeded())?;
        Ok(WorthQueryNamedClockReading::new(
            sequence,
            WorthQueryClockCoordinate::from_nanoseconds(elapsed),
        ))
    }
}

fn clock_range_exceeded() -> WorthQueryNamedClockFailure {
    WorthQueryNamedClockFailure::new(
        WorthQueryNamedClockFailureKind::ObservationFailed,
        "bank approval clock range exceeded",
    )
}

pub(crate) type BankApprovalAuthenticationOwner = Arc<
    WorthQueryInstalledAuthenticationEventOwner<
        BankSchema,
        BankApprovalClock,
        BankApprovalClockSource,
        BankApprovalVerifier,
    >,
>;

pub(crate) fn install_approval_authentication(
    schema: &WorthQueryInstalledApplicationSchema<BankSchema>,
    configuration: BankApprovalAuthenticationConfiguration,
) -> Result<BankApprovalAuthenticationOwner, WorthQueryAuthenticationEventDenial> {
    let policy = WorthQueryAuthenticationEventPolicy::new(
        Duration::from_secs(60),
        WorthQueryAuthenticationEventReuse::SingleUse,
    )
    .expect("the fixed approval event policy is valid");
    install_authentication_event_owner::<BankSchema, BankApprovalClock, _, _>(
        schema,
        BankApprovalClockSource::new()?,
        BankApprovalVerifier(configuration.verifier),
        policy,
        NonZeroUsize::new(128).expect("approval event capacity is nonzero"),
    )
    .map(Arc::new)
}
