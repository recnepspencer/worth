use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use worth_query_admission::facade::authentication_event::{
    install_authentication_event_owner, WorthQueryAuthenticationEventChallenge,
    WorthQueryAuthenticationEventFuture, WorthQueryAuthenticationEventPolicy,
    WorthQueryAuthenticationEventReuse, WorthQueryAuthenticationEventVerifier,
    WorthQueryAuthenticationEventVerifierFailure, WorthQueryInstalledAuthenticationEventOwner,
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::{
    WorthQueryClockCoordinate, WorthQueryClockSourceIdentity, WorthQueryClockTimelineIdentity,
    WorthQueryInstalledApplicationSchema, WorthQueryNamedClock, WorthQueryNamedClockFailure,
    WorthQueryNamedClockReading, WorthQueryNamedClockSource,
};

use super::super::operator_identity::operator_identity;
use super::super::schema::BoundedDimensionSchema;

pub struct CertificationAuthenticationClock;
impl WorthQueryNamedClock for CertificationAuthenticationClock {
    const PORTABLE_IDENTITY: &'static str = "certification.workflow-authentication-clock.v1";
}

pub struct CertificationAuthenticationClockSource {
    started: Instant,
    sequence: Arc<AtomicU64>,
    offset_nanoseconds: Arc<AtomicU64>,
}

impl Clone for CertificationAuthenticationClockSource {
    fn clone(&self) -> Self {
        Self {
            started: self.started,
            sequence: Arc::clone(&self.sequence),
            offset_nanoseconds: Arc::clone(&self.offset_nanoseconds),
        }
    }
}

impl CertificationAuthenticationClockSource {
    fn new() -> Self {
        Self {
            started: Instant::now(),
            sequence: Arc::new(AtomicU64::new(1)),
            offset_nanoseconds: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn advance_for_test(&self, duration: Duration) {
        let additional = u64::try_from(duration.as_nanos()).expect("finite test clock advance");
        self.offset_nanoseconds
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |previous| {
                previous.checked_add(additional)
            })
            .expect("finite test clock offset");
    }
}

impl WorthQueryNamedClockSource<CertificationAuthenticationClock>
    for CertificationAuthenticationClockSource
{
    const SEMANTIC_IDENTITY: &'static str = "certification.workflow-authentication-source.v1";

    fn source_identity(&self) -> WorthQueryClockSourceIdentity {
        WorthQueryClockSourceIdentity::declare("certification-workflow-authentication").unwrap()
    }

    fn timeline_identity(&self) -> WorthQueryClockTimelineIdentity {
        WorthQueryClockTimelineIdentity::declare("certification-process-local").unwrap()
    }

    fn observe(
        &self,
    ) -> Result<
        WorthQueryNamedClockReading<CertificationAuthenticationClock>,
        WorthQueryNamedClockFailure,
    > {
        Ok(WorthQueryNamedClockReading::new(
            self.sequence.fetch_add(1, Ordering::SeqCst),
            WorthQueryClockCoordinate::from_nanoseconds(
                u64::try_from(self.started.elapsed().as_nanos())
                    .expect("finite test clock lifetime")
                    .checked_add(self.offset_nanoseconds.load(Ordering::SeqCst))
                    .expect("finite test clock coordinate"),
            ),
        ))
    }
}

pub struct CertificationAuthenticationVerifier;
impl WorthQueryAuthenticationEventVerifier for CertificationAuthenticationVerifier {
    type Credential = ();

    fn configuration_identity(&self) -> &str {
        "certification-workflow-approval-factor-v1"
    }

    fn verify<'a>(
        &'a self,
        (): Self::Credential,
        challenge: &'a WorthQueryAuthenticationEventChallenge,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationEventFuture<'a> {
        Box::pin(async move {
            if challenge.principal() != &operator_identity()
                || !matches!(
                    challenge.intent().purpose(),
                    "workflow-approval-signature" | "certification-account-login"
                )
                || *challenge.nonce() == [0; 32]
            {
                return Err(WorthQueryAuthenticationEventVerifierFailure::CredentialRejected);
            }
            Ok(())
        })
    }
}

pub type CertificationAuthenticationOwner = Arc<
    WorthQueryInstalledAuthenticationEventOwner<
        BoundedDimensionSchema,
        CertificationAuthenticationClock,
        CertificationAuthenticationClockSource,
        CertificationAuthenticationVerifier,
    >,
>;

pub fn install_certification_authentication(
    schema: &WorthQueryInstalledApplicationSchema<BoundedDimensionSchema>,
) -> CertificationAuthenticationOwner {
    install_certification_authentication_with_clock(schema).0
}

pub fn install_certification_authentication_with_clock(
    schema: &WorthQueryInstalledApplicationSchema<BoundedDimensionSchema>,
) -> (
    CertificationAuthenticationOwner,
    CertificationAuthenticationClockSource,
) {
    let clock = CertificationAuthenticationClockSource::new();
    let owner = Arc::new(
        install_authentication_event_owner::<
            BoundedDimensionSchema,
            CertificationAuthenticationClock,
            _,
            _,
        >(
            schema,
            clock.clone(),
            CertificationAuthenticationVerifier,
            WorthQueryAuthenticationEventPolicy::new(
                Duration::from_secs(60),
                WorthQueryAuthenticationEventReuse::SingleUse,
            )
            .unwrap(),
            NonZeroUsize::new(128).unwrap(),
        )
        .unwrap(),
    );
    (owner, clock)
}
