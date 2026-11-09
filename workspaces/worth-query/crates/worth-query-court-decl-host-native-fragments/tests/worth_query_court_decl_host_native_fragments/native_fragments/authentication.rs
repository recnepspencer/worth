use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, SystemTime};
use worth_query_decl::facade::authentication::WorthQueryExternalPrincipalIdentity;

use worth_query_host::facade::admission::authenticated_principal::*;
use worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema;

use worth_query_host::facade::domain::ApplicationSchema;

pub(super) fn authenticate_external<Schema: ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    subject: &str,
    lifetime: Duration,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<Schema> {
    let adapter = admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("bank").unwrap(),
            WorthQueryAuthenticationMethod::new("test-oidc").unwrap(),
            scope.clock().clone(),
        ),
        CausalIdentityAdapter,
    )
    .unwrap();
    block_on(adapter.authenticate(
        TestCredential {
            subject: subject.to_string(),
            lifetime,
        },
        scope,
    ))
    .unwrap()
}

pub(super) fn block_on<F: Future>(future: F) -> F::Output {
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

struct TestCredential {
    subject: String,
    lifetime: Duration,
}

struct CausalIdentityAdapter;

impl WorthQueryAuthenticationAdapter for CausalIdentityAdapter {
    type Credential = TestCredential;

    fn configuration_identity(&self) -> &str {
        "identity-execution-test-adapter-v1"
    }

    fn validate<'a>(
        &'a self,
        credential: Self::Credential,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                external_identity(&credential.subject),
                WorthQueryAuthenticationAudience::new("bank").unwrap(),
                WorthQueryAuthenticationMethod::new("test-oidc").unwrap(),
                now,
                now + credential.lifetime,
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

pub(super) fn external_identity(subject: &str) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new("https://issuer.example", subject).unwrap()
}
pub(super) fn live_scope() -> WorthQueryRequestScope {
    let source = WorthQueryCancellationSource::new();
    let clock = WorthQueryAdmissionClock::system();
    WorthQueryRequestScope::new(
        Some(clock.now() + Duration::from_secs(60)),
        source.token(),
        clock,
    )
}
