use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant, SystemTime};

use worth_query_admission::facade::authenticated_principal::{
    admit_authentication_adapter, WorthQueryAuthenticatedExternalPrincipal,
    WorthQueryAuthenticationAdapter, WorthQueryAuthenticationAdapterAdmission,
    WorthQueryAuthenticationAdapterFailure, WorthQueryAuthenticationAdapterFailureKind,
    WorthQueryAuthenticationAudience, WorthQueryAuthenticationFuture,
    WorthQueryAuthenticationMethod, WorthQueryCancellationSource, WorthQueryPrincipalAttribute,
    WorthQueryRequestScope, WorthQueryValidatedExternalPrincipal,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use super::super::declaration::PublicationAuthorizationSchema;

pub(super) fn authenticate_external(
    schema: &WorthQueryInstalledApplicationSchema<PublicationAuthorizationSchema>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<PublicationAuthorizationSchema> {
    let adapter = admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("publication-proof").unwrap(),
            WorthQueryAuthenticationMethod::new("causal-test").unwrap(),
        ),
        CausalIdentityAdapter,
    )
    .unwrap();
    block_on(adapter.authenticate(TestCredential, scope)).unwrap()
}

pub(super) fn external_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new("https://publication-proof.example", "same-subject")
        .unwrap()
}

pub(super) fn request_scope() -> WorthQueryRequestScope {
    let cancellation = WorthQueryCancellationSource::new();
    WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    )
}

struct TestCredential;
struct CausalIdentityAdapter;

impl WorthQueryAuthenticationAdapter for CausalIdentityAdapter {
    type Credential = TestCredential;

    fn configuration_identity(&self) -> &str {
        "publication-authorization-proof-adapter-v1"
    }

    fn validate<'a>(
        &'a self,
        _credential: Self::Credential,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                external_identity(),
                WorthQueryAuthenticationAudience::new("publication-proof").unwrap(),
                WorthQueryAuthenticationMethod::new("causal-test").unwrap(),
                now,
                now + Duration::from_secs(60),
                vec![WorthQueryPrincipalAttribute::new("proof", "causal").unwrap()],
            )
            .map_err(|_| {
                WorthQueryAuthenticationAdapterFailure::new(
                    WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
}

fn block_on<FutureType: Future>(future: FutureType) -> FutureType::Output {
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
