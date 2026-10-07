//! Local authentication for checkpoint fixture requests.
use super::*;
use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant, SystemTime};
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;

struct LocalIdentityAdapter;
struct LocalCredential;

impl authentication::WorthQueryAuthenticationAdapter for LocalIdentityAdapter {
    type Credential = LocalCredential;

    fn configuration_identity(&self) -> &str {
        "worth.query.certification.checkpoint-authentication.v1"
    }

    fn validate<'a>(
        &'a self,
        _: Self::Credential,
        _: &'a authentication::WorthQueryRequestScope,
    ) -> authentication::WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            authentication::WorthQueryValidatedExternalPrincipal::new(
                external_identity(),
                authentication::WorthQueryAuthenticationAudience::new("checkpoint").unwrap(),
                authentication::WorthQueryAuthenticationMethod::new("local").unwrap(),
                now,
                now + Duration::from_secs(600),
                Vec::new(),
            )
            .map_err(|_| {
                authentication::WorthQueryAuthenticationAdapterFailure::new(
                    authentication::WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
}

pub(super) fn external_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new("https://checkpoint.invalid/local", "model-owner")
        .unwrap()
}

pub(in super::super) fn authenticate<Program>(
    application: &application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        Program,
    >,
) -> (
    authentication::WorthQueryRequestScope,
    authentication::WorthQueryAuthenticatedExternalPrincipal<CheckpointSchema>,
)
where
    Program: ApplicationProgramDefinition<CheckpointSchema>,
{
    let cancellation = authentication::WorthQueryCancellationSource::new();
    let scope = authentication::WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(120),
        cancellation.token(),
    );
    let adapter = authentication::admit_authentication_adapter(
        application.installed_schema(),
        authentication::WorthQueryAuthenticationAdapterAdmission::new(
            authentication::WorthQueryAuthenticationAudience::new("checkpoint").unwrap(),
            authentication::WorthQueryAuthenticationMethod::new("local").unwrap(),
        ),
        LocalIdentityAdapter,
    )
    .unwrap();
    let principal = block_on(adapter.authenticate(LocalCredential, &scope)).unwrap();
    (scope, principal)
}

fn block_on<Output>(future: impl Future<Output = Output>) -> Output {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("the local authentication completes synchronously"),
    }
}
