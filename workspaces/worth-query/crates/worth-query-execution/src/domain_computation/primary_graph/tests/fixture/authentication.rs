use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, SystemTime};

use worth_query_admission::facade::authenticated_principal::*;
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use super::external_identity;
use worth_query_installation::facade::ApplicationSchema;

pub(in crate::domain_computation::primary_graph) fn authenticate_external<
    Schema: ApplicationSchema,
>(
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
