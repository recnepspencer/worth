//! The external identity every bounded-dimension request is made under.
//!
//! Authority is not what this court is about, so one operator authenticates
//! through the ordinary admission adapter and is used for every request. The
//! adapter is real: requests resolve their application principal through the
//! installed principal binding exactly as a deployed caller's would.

use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant, SystemTime};

use worth_query_host::facade::admission::authenticated_principal::{
    admit_authentication_adapter, WorthQueryAdmittedAuthenticationAdapter,
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryAuthenticationAdapter,
    WorthQueryAuthenticationAdapterAdmission, WorthQueryAuthenticationAdapterFailure,
    WorthQueryAuthenticationAdapterFailureKind, WorthQueryAuthenticationAudience,
    WorthQueryAuthenticationFuture, WorthQueryAuthenticationMethod, WorthQueryCancellationSource,
    WorthQueryRequestScope, WorthQueryValidatedExternalPrincipal,
};
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema;

use super::schema::BoundedDimensionSchema;

pub const OPERATOR_ISSUER: &str = "https://issuer.example";
pub const OPERATOR_SUBJECT: &str = "bounded-dimension-operator";

pub struct OperatorAdapter;

impl WorthQueryAuthenticationAdapter for OperatorAdapter {
    type Credential = ();

    fn configuration_identity(&self) -> &str {
        "worth-query-bounded-dimension-operator-v1"
    }

    fn validate<'a>(
        &'a self,
        _credential: Self::Credential,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                operator_identity(),
                audience(),
                method(),
                now,
                now + Duration::from_secs(3_600),
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

pub fn operator_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(OPERATOR_ISSUER, OPERATOR_SUBJECT)
        .expect("the operator external identity is valid")
}

pub fn admit_operator_adapter(
    schema: &WorthQueryInstalledApplicationSchema<BoundedDimensionSchema>,
) -> WorthQueryAdmittedAuthenticationAdapter<BoundedDimensionSchema, OperatorAdapter> {
    admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(audience(), method()),
        OperatorAdapter,
    )
    .expect("the operator adapter must be admitted")
}

pub fn authenticate_operator(
    schema: &WorthQueryInstalledApplicationSchema<BoundedDimensionSchema>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<BoundedDimensionSchema> {
    block_on(admit_operator_adapter(schema).authenticate((), scope))
        .expect("the operator must authenticate")
}

/// A bounded request scope. Every court step completes well inside it, so a
/// hung step fails by deadline instead of waiting forever.
pub fn request_scope() -> WorthQueryRequestScope {
    let cancellation = WorthQueryCancellationSource::new();
    WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(30),
        cancellation.token(),
    )
}

fn audience() -> WorthQueryAuthenticationAudience {
    WorthQueryAuthenticationAudience::new("bounded-dimension-host")
        .expect("the operator audience is valid")
}

fn method() -> WorthQueryAuthenticationMethod {
    WorthQueryAuthenticationMethod::new("certification").expect("the operator method is valid")
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
