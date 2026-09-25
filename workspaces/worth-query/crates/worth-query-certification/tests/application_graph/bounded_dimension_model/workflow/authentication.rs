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
    sequence: AtomicU64,
}

impl CertificationAuthenticationClockSource {
    fn new() -> Self {
        Self {
            started: Instant::now(),
            sequence: AtomicU64::new(1),
        }
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
            WorthQueryClockCoordinate::from_nanoseconds(self.started.elapsed().as_nanos() as u64),
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
                || challenge.intent().purpose() != "workflow-approval-signature"
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
    Arc::new(
        install_authentication_event_owner::<
            BoundedDimensionSchema,
            CertificationAuthenticationClock,
            _,
            _,
        >(
            schema,
            CertificationAuthenticationClockSource::new(),
            CertificationAuthenticationVerifier,
            WorthQueryAuthenticationEventPolicy::new(
                Duration::from_secs(60),
                WorthQueryAuthenticationEventReuse::SingleUse,
            )
            .unwrap(),
            NonZeroUsize::new(128).unwrap(),
        )
        .unwrap(),
    )
}
