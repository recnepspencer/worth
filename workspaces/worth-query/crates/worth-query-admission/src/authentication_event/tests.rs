use std::future::Future;
use std::num::NonZeroUsize;
use std::pin::pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant, SystemTime};

use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::facade::application_schema::{
    ApplicationEntityMarkerIdentity, ApplicationEntityRef, ApplicationSchema,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_installation::facade::{
    WorthQueryClockCoordinate, WorthQueryClockSourceIdentity, WorthQueryClockTimelineIdentity,
    WorthQueryNamedClock, WorthQueryNamedClockFailure, WorthQueryNamedClockReading,
    WorthQueryNamedClockSource,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

use crate::authenticated_principal::{
    admit_authentication_adapter, WorthQueryAuthenticatedExternalPrincipal,
    WorthQueryAuthenticationAdapter, WorthQueryAuthenticationAdapterAdmission,
    WorthQueryAuthenticationAudience, WorthQueryAuthenticationFuture,
    WorthQueryAuthenticationMethod, WorthQueryCancellationSource, WorthQueryRequestScope,
    WorthQueryValidatedExternalPrincipal,
};

use super::*;

mod lifecycle;

struct TestSchema;
struct Principal;

impl ApplicationEntityMarkerIdentity<TestSchema> for Principal {
    const IDENTIFIER: &'static str = "Principal";
}

impl ApplicationSchema for TestSchema {
    const OWNER: &'static str = "authentication-event-test";
    const NAME: &'static str = "TestSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .entity(ApplicationEntityRef::<Self, Principal>::from_schema_identifier("Principal"))
            .build()
    }
}

struct PrincipalAdapter;

impl WorthQueryAuthenticationAdapter for PrincipalAdapter {
    type Credential = &'static str;

    fn configuration_identity(&self) -> &str {
        "event-test-principal-adapter"
    }

    fn validate<'a>(
        &'a self,
        credential: Self::Credential,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            Ok(WorthQueryValidatedExternalPrincipal::new(
                WorthQueryExternalPrincipalIdentity::new("issuer", credential).unwrap(),
                WorthQueryAuthenticationAudience::new("approval").unwrap(),
                WorthQueryAuthenticationMethod::new("test-factor").unwrap(),
                now - Duration::from_secs(1),
                now + Duration::from_secs(60),
                vec![],
            )
            .unwrap())
        })
    }
}

struct AuthenticationClock;

impl WorthQueryNamedClock for AuthenticationClock {
    const PORTABLE_IDENTITY: &'static str = "test.authentication-clock.v1";
}

#[derive(Clone)]
struct ClockSource {
    now: Arc<AtomicU64>,
    sequence: Arc<AtomicU64>,
    changed_identity: Arc<AtomicBool>,
}

impl ClockSource {
    fn new() -> Self {
        Self {
            now: Arc::new(AtomicU64::new(1_000)),
            sequence: Arc::new(AtomicU64::new(1)),
            changed_identity: Arc::new(AtomicBool::new(false)),
        }
    }

    fn advance(&self, nanoseconds: u64) {
        self.now.fetch_add(nanoseconds, Ordering::SeqCst);
    }
}

impl WorthQueryNamedClockSource<AuthenticationClock> for ClockSource {
    const SEMANTIC_IDENTITY: &'static str = "test.clock-source.v1";

    fn source_identity(&self) -> WorthQueryClockSourceIdentity {
        WorthQueryClockSourceIdentity::declare("source-a").unwrap()
    }

    fn timeline_identity(&self) -> WorthQueryClockTimelineIdentity {
        let identity = if self.changed_identity.load(Ordering::SeqCst) {
            "timeline-b"
        } else {
            "timeline-a"
        };
        WorthQueryClockTimelineIdentity::declare(identity).unwrap()
    }

    fn observe(
        &self,
    ) -> Result<WorthQueryNamedClockReading<AuthenticationClock>, WorthQueryNamedClockFailure> {
        Ok(WorthQueryNamedClockReading::new(
            self.sequence.fetch_add(1, Ordering::SeqCst),
            WorthQueryClockCoordinate::from_nanoseconds(self.now.load(Ordering::SeqCst)),
        ))
    }
}

#[derive(Clone)]
enum EventCredential {
    Valid,
    Rejected,
    Pending,
    WaitFor(Arc<AtomicBool>),
}

struct EventVerifier {
    observed_nonces: Arc<Mutex<Vec<[u8; 32]>>>,
}

impl WorthQueryAuthenticationEventVerifier for EventVerifier {
    type Credential = EventCredential;

    fn configuration_identity(&self) -> &str {
        "installed-event-verifier"
    }

    fn verify<'a>(
        &'a self,
        credential: Self::Credential,
        challenge: &'a WorthQueryAuthenticationEventChallenge,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationEventFuture<'a> {
        Box::pin(async move {
            self.observed_nonces
                .lock()
                .unwrap()
                .push(*challenge.nonce());
            assert_eq!(challenge.principal().subject(), "alice");
            assert_eq!(challenge.intent().purpose(), "approve-payment");
            match credential {
                EventCredential::Valid => Ok(()),
                EventCredential::Rejected => {
                    Err(WorthQueryAuthenticationEventVerifierFailure::CredentialRejected)
                }
                EventCredential::Pending => std::future::pending().await,
                EventCredential::WaitFor(release) => {
                    std::future::poll_fn(|_| {
                        if release.load(Ordering::SeqCst) {
                            Poll::Ready(Ok(()))
                        } else {
                            Poll::Pending
                        }
                    })
                    .await
                }
            }
        })
    }
}

fn installed_schema(
) -> worth_query_installation::facade::WorthQueryInstalledApplicationSchema<TestSchema> {
    let declaration = TestSchema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "authentication-event-test",
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    let index = worth_query_installation::facade::WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap();
    index.bind_application_schema(declaration).unwrap()
}

fn scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        WorthQueryCancellationSource::new().token(),
    )
}

fn intent(purpose: &str, coverage: u8) -> WorthQueryAuthenticationEventIntent {
    WorthQueryAuthenticationEventIntent::new(
        purpose,
        CanonicalDigestId::new([11; 32]),
        CanonicalDigestId::new([coverage; 32]),
    )
    .unwrap()
}

fn principal_named(
    schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<TestSchema>,
    name: &'static str,
) -> WorthQueryAuthenticatedExternalPrincipal<TestSchema> {
    let adapter = admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("approval").unwrap(),
            WorthQueryAuthenticationMethod::new("test-factor").unwrap(),
        ),
        PrincipalAdapter,
    )
    .unwrap();
    block_on(adapter.authenticate(name, &scope())).unwrap()
}

fn principal(
    schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<TestSchema>,
) -> WorthQueryAuthenticatedExternalPrincipal<TestSchema> {
    principal_named(schema, "alice")
}

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

#[test]
fn admitted_event_binds_exact_intent_principal_and_single_use() {
    let schema = installed_schema();
    let principal = principal(&schema);
    let source = ClockSource::new();
    let nonces = Arc::new(Mutex::new(Vec::new()));
    let policy = WorthQueryAuthenticationEventPolicy::new(
        Duration::from_nanos(100),
        WorthQueryAuthenticationEventReuse::SingleUse,
    )
    .unwrap();
    let owner = install_authentication_event_owner::<TestSchema, AuthenticationClock, _, _>(
        &schema,
        source,
        EventVerifier {
            observed_nonces: Arc::clone(&nonces),
        },
        policy,
        NonZeroUsize::new(2).unwrap(),
    )
    .unwrap();
    let intended = intent("approve-payment", 7);
    let event = block_on(owner.authenticate(
        EventCredential::Valid,
        &principal,
        intended.clone(),
        &scope(),
    ))
    .ok()
    .unwrap();
    assert_eq!(event.issued_at_nanoseconds(), 1_000);
    assert_eq!(event.policy(), policy);
    assert_eq!(
        owner
            .consume_for_signing(&event, &principal, &intent("other-purpose", 7), &scope())
            .err(),
        Some(WorthQueryAuthenticationEventDenial::IntentMismatch)
    );
    assert_eq!(
        owner
            .consume_for_signing(&event, &principal, &intent("approve-payment", 8), &scope())
            .err(),
        Some(WorthQueryAuthenticationEventDenial::IntentMismatch)
    );
    let different_signing_intent = WorthQueryAuthenticationEventIntent::new(
        "approve-payment",
        CanonicalDigestId::new([12; 32]),
        CanonicalDigestId::new([7; 32]),
    )
    .unwrap();
    assert_eq!(
        owner
            .consume_for_signing(&event, &principal, &different_signing_intent, &scope())
            .err(),
        Some(WorthQueryAuthenticationEventDenial::IntentMismatch)
    );
    let another_principal = principal_named(&schema, "bob");
    assert_eq!(
        owner
            .consume_for_signing(&event, &another_principal, &intended, &scope())
            .err(),
        Some(WorthQueryAuthenticationEventDenial::PrincipalMismatch)
    );
    let proof = owner
        .consume_for_signing(&event, &principal, &intended, &scope())
        .ok()
        .unwrap();
    assert_eq!(proof.intent(), &intended);
    assert_eq!(proof.policy(), policy);
    assert_eq!(proof.principal(), principal.identity());
    let proof = owner
        .readmit_consumed_for_signing(proof, &principal, &intended, &scope())
        .ok()
        .unwrap();
    let foreign_owner =
        install_authentication_event_owner::<TestSchema, AuthenticationClock, _, _>(
            &schema,
            ClockSource::new(),
            EventVerifier {
                observed_nonces: Arc::new(Mutex::new(Vec::new())),
            },
            policy,
            NonZeroUsize::new(2).unwrap(),
        )
        .unwrap();
    assert_eq!(
        foreign_owner
            .readmit_consumed_for_signing(proof, &principal, &intended, &scope())
            .err(),
        Some(WorthQueryAuthenticationEventDenial::WrongOwner)
    );
    assert_eq!(
        owner
            .consume_for_signing(&event, &principal, &intended, &scope())
            .err(),
        Some(WorthQueryAuthenticationEventDenial::EventUnavailable)
    );
    assert_eq!(nonces.lock().unwrap().len(), 1);
}
