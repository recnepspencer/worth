use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant, SystemTime};

use worth_query_host::facade::{
    admission::authenticated_principal as authentication,
    declaration::authentication::WorthQueryExternalPrincipalIdentity,
    domain::WorthQueryInstalledApplicationSchema,
};

use crate::ConsumerSchema;

pub(super) struct LocalIdentityAdapter;

pub(super) struct LocalCredential(&'static str);

impl LocalCredential {
    pub(super) fn issued_for_model_owner() -> Self {
        Self("model-owner")
    }
}

impl authentication::WorthQueryAuthenticationAdapter for LocalIdentityAdapter {
    type Credential = LocalCredential;

    fn configuration_identity(&self) -> &str {
        "worth.query.consumer.local-authentication.v1"
    }

    fn validate<'a>(
        &'a self,
        credential: Self::Credential,
        _scope: &'a authentication::WorthQueryRequestScope,
    ) -> authentication::WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            if credential.0 != "model-owner" {
                return Err(protocol_failure());
            }
            let now = SystemTime::now();
            authentication::WorthQueryValidatedExternalPrincipal::new(
                external_identity(),
                authentication::WorthQueryAuthenticationAudience::new("consumer-root").unwrap(),
                authentication::WorthQueryAuthenticationMethod::new("local-credential").unwrap(),
                now,
                now + Duration::from_secs(600),
                Vec::new(),
            )
            .map_err(|_| protocol_failure())
        })
    }
}

fn protocol_failure() -> authentication::WorthQueryAuthenticationAdapterFailure {
    authentication::WorthQueryAuthenticationAdapterFailure::new(
        authentication::WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
    )
}

pub(super) fn external_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new("https://consumer.invalid/local", "model-owner")
        .unwrap()
}

pub(super) fn admit(
    schema: &WorthQueryInstalledApplicationSchema<ConsumerSchema>,
) -> authentication::WorthQueryAdmittedAuthenticationAdapter<ConsumerSchema, LocalIdentityAdapter> {
    authentication::admit_authentication_adapter(
        schema,
        authentication::WorthQueryAuthenticationAdapterAdmission::new(
            authentication::WorthQueryAuthenticationAudience::new("consumer-root").unwrap(),
            authentication::WorthQueryAuthenticationMethod::new("local-credential").unwrap(),
        ),
        LocalIdentityAdapter,
    )
    .expect("the local adapter is admitted against the installed consumer schema")
}

pub(super) fn request_scope() -> authentication::WorthQueryRequestScope {
    let cancellation = authentication::WorthQueryCancellationSource::new();
    authentication::WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(120),
        cancellation.token(),
    )
}

pub(super) fn block_on<Output>(future: impl Future<Output = Output>) -> Output {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("the local authentication provider completes synchronously"),
    }
}
