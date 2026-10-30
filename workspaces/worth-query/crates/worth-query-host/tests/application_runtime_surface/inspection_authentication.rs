//! Actual admitted authentication for the Host identity fixture.
use super::HostIdentitySchema;
use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
    time::{Duration, SystemTime},
};
use worth_query_decl::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_host::facade::{
    admission::authenticated_principal::*, domain::WorthQueryInstalledApplicationSchema,
};

pub(super) struct IdentityAdapter;
impl WorthQueryAuthenticationAdapter for IdentityAdapter {
    type Credential = ();
    fn configuration_identity(&self) -> &str {
        "worth.query.host.inspection.authentication.v1"
    }
    fn validate<'a>(
        &'a self,
        _: (),
        _: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "subject")
                    .unwrap(),
                WorthQueryAuthenticationAudience::new("host").unwrap(),
                WorthQueryAuthenticationMethod::new("test").unwrap(),
                now,
                now + Duration::from_secs(3600),
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

pub(super) fn authenticate(
    schema: &WorthQueryInstalledApplicationSchema<HostIdentitySchema>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<HostIdentitySchema> {
    let adapter = admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("host").unwrap(),
            WorthQueryAuthenticationMethod::new("test").unwrap(),
        ),
        IdentityAdapter,
    )
    .unwrap();
    let mut future = pin!(adapter.authenticate((), scope));
    let mut context = Context::from_waker(Waker::noop());
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output.unwrap(),
            Poll::Pending => std::thread::yield_now(),
        }
    }
}
