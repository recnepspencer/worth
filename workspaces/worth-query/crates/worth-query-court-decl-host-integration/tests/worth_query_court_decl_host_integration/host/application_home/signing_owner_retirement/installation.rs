//! A real signing verifier, with observation of its final supplier destruction.
use super::*;

pub(super) type ObservedOwner = WorthQueryInstalledAuthenticationEventOwner<
    fixture::schema::HomeIdentitySchema,
    CertificationAuthenticationClock,
    CertificationAuthenticationClockSource,
    ObservedVerifier,
>;

pub(super) struct ObservedVerifier {
    drops: Arc<AtomicUsize>,
    verifier: CertificationAuthenticationVerifier,
}
impl Drop for ObservedVerifier {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
impl WorthQueryAuthenticationEventVerifier for ObservedVerifier {
    type Credential = ();
    fn configuration_identity(&self) -> &str {
        self.verifier.configuration_identity()
    }
    fn verify<'a>(
        &'a self,
        credential: (),
        challenge: &'a WorthQueryAuthenticationEventChallenge,
        scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationEventFuture<'a> {
        self.verifier.verify(credential, challenge, scope)
    }
}

pub(super) fn install(
    schema: &WorthQueryInstalledApplicationSchema<fixture::schema::HomeIdentitySchema>,
    drops: Arc<AtomicUsize>,
) -> Arc<ObservedOwner> {
    Arc::new(
        install_authentication_event_owner::<_, CertificationAuthenticationClock, _, _>(
            schema,
            CertificationAuthenticationClockSource::new(),
            ObservedVerifier {
                drops,
                verifier: CertificationAuthenticationVerifier,
            },
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
