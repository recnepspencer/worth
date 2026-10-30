//! The ordinary public owner authenticates each actual prepared signing intent.
pub use super::super::{operator_identity::operator_identity, schema::Schema, world::App};
pub use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
pub use worth_query_host::facade::{
    admission::{authenticated_principal::WorthQueryRequestScope, authentication_event::*},
    application_installation::WorthQueryProgramOwner,
    domain::*,
};
pub struct Clock;
impl WorthQueryNamedClock for Clock {
    const PORTABLE_IDENTITY: &'static str = "worth.query.signed-path.clock.v1";
}
pub struct Source {
    pub started: Instant,
    pub sequence: AtomicU64,
}
impl WorthQueryNamedClockSource<Clock> for Source {
    const SEMANTIC_IDENTITY: &'static str = "worth.query.signed-path.monotonic.v1";
    fn source_identity(&self) -> WorthQueryClockSourceIdentity {
        WorthQueryClockSourceIdentity::declare("signed-path-clock").unwrap()
    }
    fn timeline_identity(&self) -> WorthQueryClockTimelineIdentity {
        WorthQueryClockTimelineIdentity::declare("signed-path-process").unwrap()
    }
    fn observe(&self) -> Result<WorthQueryNamedClockReading<Clock>, WorthQueryNamedClockFailure> {
        let denied = || {
            WorthQueryNamedClockFailure::new(
                WorthQueryNamedClockFailureKind::ObservationFailed,
                "clock exhausted",
            )
        };
        let coordinate = u64::try_from(self.started.elapsed().as_nanos()).map_err(|_| denied())?;
        let sequence = {
            let update_counter = &self.sequence;
            let mut value = update_counter.load(Ordering::Relaxed);
            loop {
                let Some(updated_value) = value.checked_add(1) else {
                    break Err(value);
                };
                match update_counter.compare_exchange_weak(
                    value,
                    updated_value,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => value = reloaded_value,
                }
            }
        }
        .map_err(|_| denied())?;
        Ok(WorthQueryNamedClockReading::new(
            sequence,
            WorthQueryClockCoordinate::from_nanoseconds(coordinate),
        ))
    }
}
pub struct Verifier;
impl WorthQueryAuthenticationEventVerifier for Verifier {
    type Credential = &'static str;
    fn configuration_identity(&self) -> &'static str {
        "signed-path-verifier.v1"
    }
    fn verify<'a>(
        &'a self,
        credential: &'static str,
        challenge: &'a WorthQueryAuthenticationEventChallenge,
        _: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationEventFuture<'a> {
        Box::pin(async move {
            if credential == "verify"
                && challenge.principal() == &operator_identity()
                && challenge.intent().purpose() == "path-verification"
            {
                Ok(())
            } else {
                Err(WorthQueryAuthenticationEventVerifierFailure::CredentialRejected)
            }
        })
    }
}
pub type Owner = WorthQueryInstalledAuthenticationEventOwner<Schema, Clock, Source, Verifier>;
pub fn install(app: &mut App) -> Arc<Owner> {
    let owner = Arc::new(
        install_authentication_event_owner::<Schema, Clock, _, _>(
            app.installed_schema(),
            Source {
                started: Instant::now(),
                sequence: AtomicU64::new(1),
            },
            Verifier,
            WorthQueryAuthenticationEventPolicy::new(
                Duration::from_secs(60),
                WorthQueryAuthenticationEventReuse::SingleUse,
            )
            .unwrap(),
            NonZeroUsize::new(16).unwrap(),
        )
        .unwrap(),
    );
    app.install_signing_owner(*app.owned_revision(), owner.signing_owner())
        .unwrap();
    owner
}
