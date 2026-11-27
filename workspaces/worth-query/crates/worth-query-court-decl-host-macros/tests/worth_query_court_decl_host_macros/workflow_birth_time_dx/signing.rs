//! The authentication-event owner a workflow Home requires for approvals.
//! This workflow has no approval node, so its verifier accepts no factor;
//! it exists only to give `retain_workflow_spec` its signing owner.
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::admission::authentication_event::{
    install_authentication_event_owner, WorthQueryAuthenticationEventChallenge,
    WorthQueryAuthenticationEventFuture, WorthQueryAuthenticationEventPolicy,
    WorthQueryAuthenticationEventReuse, WorthQueryAuthenticationEventVerifier,
    WorthQueryAuthenticationEventVerifierFailure, WorthQueryInstalledAuthenticationEventOwner,
};
use worth_query_host::facade::domain::{
    WorthQueryClockCoordinate, WorthQueryClockSourceIdentity, WorthQueryClockTimelineIdentity,
    WorthQueryInstalledApplicationSchema, WorthQueryNamedClock, WorthQueryNamedClockFailure,
    WorthQueryNamedClockReading, WorthQueryNamedClockSource,
};

use super::schema::BirthSchema;

pub struct SigningClock;
impl WorthQueryNamedClock for SigningClock {
    const PORTABLE_IDENTITY: &'static str = "workflow-birth-time.dx.signing-clock.v1";
}

/// A process-local monotone clock: every reading advances by one tick.
#[derive(Clone, Default)]
pub struct SigningClockSource(Arc<AtomicU64>);

impl WorthQueryNamedClockSource<SigningClock> for SigningClockSource {
    const SEMANTIC_IDENTITY: &'static str = "workflow-birth-time.dx.signing-source.v1";

    fn source_identity(&self) -> WorthQueryClockSourceIdentity {
        WorthQueryClockSourceIdentity::declare("workflow-birth-time-signing").unwrap()
    }

    fn timeline_identity(&self) -> WorthQueryClockTimelineIdentity {
        WorthQueryClockTimelineIdentity::declare("workflow-birth-time-process").unwrap()
    }

    fn observe(
        &self,
    ) -> Result<WorthQueryNamedClockReading<SigningClock>, WorthQueryNamedClockFailure> {
        let tick = self.0.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(WorthQueryNamedClockReading::new(
            tick,
            WorthQueryClockCoordinate::from_nanoseconds(tick),
        ))
    }
}

/// Accepts no credential: nothing in this workflow is signed.
pub struct NoFactor;
impl WorthQueryAuthenticationEventVerifier for NoFactor {
    type Credential = ();

    fn configuration_identity(&self) -> &str {
        "workflow-birth-time-no-factor-v1"
    }

    fn verify<'a>(
        &'a self,
        (): Self::Credential,
        _challenge: &'a WorthQueryAuthenticationEventChallenge,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationEventFuture<'a> {
        Box::pin(async { Err(WorthQueryAuthenticationEventVerifierFailure::CredentialRejected) })
    }
}

pub type SigningOwner = Arc<
    WorthQueryInstalledAuthenticationEventOwner<
        BirthSchema,
        SigningClock,
        SigningClockSource,
        NoFactor,
    >,
>;

pub fn install(schema: &WorthQueryInstalledApplicationSchema<BirthSchema>) -> SigningOwner {
    Arc::new(
        install_authentication_event_owner::<BirthSchema, SigningClock, _, _>(
            schema,
            SigningClockSource::default(),
            NoFactor,
            WorthQueryAuthenticationEventPolicy::new(
                Duration::from_secs(60),
                WorthQueryAuthenticationEventReuse::SingleUse,
            )
            .unwrap(),
            NonZeroUsize::new(16).unwrap(),
        )
        .expect("the signing owner installs"),
    )
}
