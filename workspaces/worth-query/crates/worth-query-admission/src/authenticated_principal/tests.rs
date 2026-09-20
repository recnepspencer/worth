use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant, SystemTime};

use worth_query_declaration::facade::application_schema::{
    ApplicationEntityMarkerIdentity, ApplicationEntityRef, ApplicationSchema,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

use super::*;

struct TestSchema;
struct Principal;

impl ApplicationEntityMarkerIdentity<TestSchema> for Principal {
    const IDENTIFIER: &'static str = "Principal";
}

impl ApplicationSchema for TestSchema {
    const OWNER: &'static str = "authentication-test";
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

enum TestCredential {
    Valid,
    WrongAudience,
    Expired,
}

struct CausalAdapter;

impl WorthQueryAuthenticationAdapter for CausalAdapter {
    type Credential = TestCredential;

    fn configuration_identity(&self) -> &str {
        "causal-adapter-v1"
    }

    fn validate<'a>(
        &'a self,
        credential: Self::Credential,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            let (audience, expires_at) = match credential {
                TestCredential::Valid => ("bank", now + Duration::from_secs(60)),
                TestCredential::WrongAudience => ("other", now + Duration::from_secs(60)),
                TestCredential::Expired => ("bank", now - Duration::from_secs(1)),
            };
            WorthQueryValidatedExternalPrincipal::new(
                WorthQueryExternalPrincipalIdentity::new("https://issuer", "subject").unwrap(),
                WorthQueryAuthenticationAudience::new(audience).unwrap(),
                WorthQueryAuthenticationMethod::new("causal-test").unwrap(),
                now - Duration::from_secs(2),
                expires_at,
                vec![WorthQueryPrincipalAttribute::new("display", "Test User").unwrap()],
            )
            .map_err(|_| {
                WorthQueryAuthenticationAdapterFailure::new(
                    WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
}

struct ConfiguredAdapter(&'static str);

impl WorthQueryAuthenticationAdapter for ConfiguredAdapter {
    type Credential = ();

    fn configuration_identity(&self) -> &str {
        self.0
    }

    fn validate<'a>(
        &'a self,
        _credential: Self::Credential,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async {
            Err(WorthQueryAuthenticationAdapterFailure::new(
                WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
            ))
        })
    }
}

#[test]
fn only_admitted_adapter_output_mints_external_proof() {
    let schema = installed_schema();
    let adapter = admit_authentication_adapter(
        &schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("bank").unwrap(),
            WorthQueryAuthenticationMethod::new("causal-test").unwrap(),
        ),
        CausalAdapter,
    )
    .unwrap();
    let scope = live_scope();
    let proof = block_on(adapter.authenticate(TestCredential::Valid, &scope)).unwrap();
    assert_eq!(proof.identity().issuer(), "https://issuer");
    assert_eq!(proof.identity().subject(), "subject");
    assert_eq!(
        proof.binding_identity().runtime_ordinal(),
        schema.binding_identity().runtime_ordinal()
    );
    assert!(!proof.is_expired());
    assert_eq!(proof.attributes().len(), 1);
}

#[test]
fn mismatched_expired_cancelled_and_timed_out_candidates_fail_closed() {
    let schema = installed_schema();
    let adapter = admit_authentication_adapter(
        &schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("bank").unwrap(),
            WorthQueryAuthenticationMethod::new("causal-test").unwrap(),
        ),
        CausalAdapter,
    )
    .unwrap();

    assert_eq!(
        block_on(adapter.authenticate(TestCredential::WrongAudience, &live_scope()))
            .unwrap_err()
            .kind(),
        WorthQueryAuthenticationDenialKind::AudienceMismatch
    );
    assert_eq!(
        block_on(adapter.authenticate(TestCredential::Expired, &live_scope()))
            .unwrap_err()
            .kind(),
        WorthQueryAuthenticationDenialKind::Expired
    );

    let cancelled = WorthQueryCancellationSource::new();
    cancelled.cancel();
    let cancelled_scope =
        WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(60), cancelled.token());
    assert_eq!(
        block_on(adapter.authenticate(TestCredential::Valid, &cancelled_scope))
            .unwrap_err()
            .kind(),
        WorthQueryAuthenticationDenialKind::Cancelled
    );

    let source = WorthQueryCancellationSource::new();
    let expired_scope =
        WorthQueryRequestScope::new(Instant::now() - Duration::from_secs(1), source.token());
    assert_eq!(
        block_on(adapter.authenticate(TestCredential::Valid, &expired_scope))
            .unwrap_err()
            .kind(),
        WorthQueryAuthenticationDenialKind::DeadlineExceeded
    );
}

#[test]
fn cancellation_future_is_woken_by_its_source() {
    let source = WorthQueryCancellationSource::new();
    let token = source.token();
    let mut future = pin!(token.cancelled());
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    assert_eq!(future.as_mut().poll(&mut context), Poll::Pending);
    source.cancel();
    assert_eq!(future.as_mut().poll(&mut context), Poll::Ready(()));
}

#[test]
fn proof_debug_discloses_no_external_identity_or_attribute_values() {
    let schema = installed_schema();
    let adapter = admit_authentication_adapter(
        &schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("bank").unwrap(),
            WorthQueryAuthenticationMethod::new("causal-test").unwrap(),
        ),
        CausalAdapter,
    )
    .unwrap();
    let proof = block_on(adapter.authenticate(TestCredential::Valid, &live_scope())).unwrap();
    let debug = format!("{proof:?}");
    assert!(!debug.contains("Test User"));
    assert!(!debug.contains("https://issuer"));
    assert!(!debug.contains("subject"));
}

#[test]
fn adapter_identity_converges_and_separates_every_semantic_binding_axis() {
    let schema = installed_schema();
    let admit = |configuration, audience, method| {
        admit_authentication_adapter(
            &schema,
            WorthQueryAuthenticationAdapterAdmission::new(
                WorthQueryAuthenticationAudience::new(audience).unwrap(),
                WorthQueryAuthenticationMethod::new(method).unwrap(),
            ),
            ConfiguredAdapter(configuration),
        )
        .unwrap()
        .adapter_identity()
        .to_owned()
    };

    let baseline = admit("configuration-a", "bank", "oidc");
    assert_eq!(baseline, admit("configuration-a", "bank", "oidc"));
    assert_ne!(baseline, admit("configuration-b", "bank", "oidc"));
    assert_ne!(baseline, admit("configuration-a", "other", "oidc"));
    assert_ne!(baseline, admit("configuration-a", "bank", "mtls"));
}

#[test]
fn adapter_identity_uses_foundational_digest_without_private_sha_or_type_names() {
    let admission = include_str!("admission.rs");
    let identity = include_str!("adapter_identity.rs");

    assert!(!admission.contains("Sha256"));
    assert!(!admission.contains("type_name"));
    assert!(!identity.contains("use sha2"));
    assert!(!identity.contains("type_name"));
    assert!(identity.contains("CanonicalDigestAlgorithmId::sha256()"));
    assert!(identity.contains("prepare_canonical_basis_sequence"));
}

fn installed_schema(
) -> worth_query_installation::facade::WorthQueryInstalledApplicationSchema<TestSchema> {
    let declaration = TestSchema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "authentication-test",
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

fn live_scope() -> WorthQueryRequestScope {
    let source = WorthQueryCancellationSource::new();
    WorthQueryRequestScope::new(Instant::now() + Duration::from_secs(60), source.token())
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
