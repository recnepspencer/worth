use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant, SystemTime};

use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticationAdapter, WorthQueryAuthenticationAdapterFailure,
    WorthQueryAuthenticationAdapterFailureKind, WorthQueryAuthenticationAudience,
    WorthQueryAuthenticationFuture, WorthQueryAuthenticationMethod, WorthQueryCancellationSource,
    WorthQueryRequestScope, WorthQueryValidatedExternalPrincipal,
};
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;

use crate::BankAuthenticationConfiguration;

pub(super) fn external_identity(
    scenario: &str,
    subject: &str,
) -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(
        format!("https://{scenario}.bank.test.invalid"),
        subject,
    )
    .unwrap()
}

pub(crate) fn request_scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        WorthQueryCancellationSource::new().token(),
    )
}

pub(super) fn authentication_configuration() -> BankAuthenticationConfiguration {
    BankAuthenticationConfiguration::new(
        WorthQueryAuthenticationAudience::new("bank-phase-7-capability-test").unwrap(),
        WorthQueryAuthenticationMethod::new("causal-phase-7-adapter").unwrap(),
    )
}

pub(super) struct TestCredential(pub(super) WorthQueryExternalPrincipalIdentity);
pub(super) struct TestAuthenticationAdapter;

impl WorthQueryAuthenticationAdapter for TestAuthenticationAdapter {
    type Credential = TestCredential;

    fn configuration_identity(&self) -> &str {
        "bank-phase-7-capability-adapter-v1"
    }

    fn validate<'a>(
        &'a self,
        credential: Self::Credential,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                credential.0,
                WorthQueryAuthenticationAudience::new("bank-phase-7-capability-test").unwrap(),
                WorthQueryAuthenticationMethod::new("causal-phase-7-adapter").unwrap(),
                now,
                now + Duration::from_secs(60),
                Vec::new(),
            )
            .map_err(|_| {
                WorthQueryAuthenticationAdapterFailure::new(
                    WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
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
