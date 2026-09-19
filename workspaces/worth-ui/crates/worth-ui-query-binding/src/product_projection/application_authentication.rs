use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant, SystemTime};

use worth_query_host::facade::{
    admission::authenticated_principal as authentication,
    domain::WorthQueryInstalledApplicationSchema,
};

use crate::declaration::WorthUiApplicationSchema;

use super::application_runtime::external_identity;
use super::WorthUiStatusOwnerError;

pub(super) struct WorthUiLocalSourceAdapter;

pub(super) struct WorthUiLocalSourceCredential;

impl authentication::WorthQueryAuthenticationAdapter for WorthUiLocalSourceAdapter {
    type Credential = WorthUiLocalSourceCredential;

    fn configuration_identity(&self) -> &str {
        "worth.ui.local-status-source.v1"
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
                authentication::WorthQueryAuthenticationAudience::new("worth-ui-platform-pulse")
                    .expect("the local UI audience is canonical"),
                authentication::WorthQueryAuthenticationMethod::new("local-source-owner")
                    .expect("the local UI method is canonical"),
                now,
                now + Duration::from_secs(120),
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

pub(super) fn request_scope() -> authentication::WorthQueryRequestScope {
    let cancellation = authentication::WorthQueryCancellationSource::new();
    authentication::WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(120),
        cancellation.token(),
    )
}

pub(super) fn authenticate(
    schema: &WorthQueryInstalledApplicationSchema<WorthUiApplicationSchema>,
    scope: &authentication::WorthQueryRequestScope,
) -> Result<
    authentication::WorthQueryAuthenticatedExternalPrincipal<WorthUiApplicationSchema>,
    WorthUiStatusOwnerError,
> {
    let adapter = authentication::admit_authentication_adapter(
        schema,
        authentication::WorthQueryAuthenticationAdapterAdmission::new(
            authentication::WorthQueryAuthenticationAudience::new("worth-ui-platform-pulse")
                .expect("the local UI audience is canonical"),
            authentication::WorthQueryAuthenticationMethod::new("local-source-owner")
                .expect("the local UI method is canonical"),
        ),
        WorthUiLocalSourceAdapter,
    )
    .map_err(|error| WorthUiStatusOwnerError::Authentication(format!("{error:?}")))?;
    block_on(adapter.authenticate(WorthUiLocalSourceCredential, scope))
        .map_err(|error| WorthUiStatusOwnerError::Authentication(format!("{error:?}")))
}

fn block_on<Output>(future: impl Future<Output = Output>) -> Output {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("the local UI identity adapter completes synchronously"),
    }
}
