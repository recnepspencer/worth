//! The external identity every document-retention request is made under.
//!
//! Authority is not what this court is about, so one operator authenticates
//! through the ordinary admission adapter and is used for every request. The
//! adapter is real: requests resolve their application principal through the
//! installed principal binding exactly as a deployed caller's would.

pub use std::future::Future;
pub use std::pin::pin;
pub use std::task::{Context, Poll, Waker};
pub use std::time::Duration;

pub use worth_query_host::facade::admission::authenticated_principal::{
    admit_authentication_adapter, WorthQueryAdmittedAuthenticationAdapter,
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryAuthenticationAdapter,
    WorthQueryAuthenticationAdapterAdmission, WorthQueryAuthenticationAdapterFailure,
    WorthQueryAuthenticationAdapterFailureKind, WorthQueryAuthenticationAudience,
    WorthQueryAuthenticationFuture, WorthQueryAuthenticationMethod, WorthQueryCancellationSource,
    WorthQueryRequestScope, WorthQueryValidatedExternalPrincipal,
};
pub use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;
pub use worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema;

pub use worth_query_host::facade::declaration::application_schema::ApplicationSchema;

worth_query_decl::facade::worth_query_authentication_method!(CertificationMethod);
worth_query_decl::facade::worth_query_authentication_audience!(DocumentRetentionHostAudience);

worth_query_decl::facade::worth_query_principal_issuer!(pub OperatorIssuer);
pub const OPERATOR_SUBJECT: &str = "document-retention-operator";

pub struct OperatorAdapter;

impl WorthQueryAuthenticationAdapter for OperatorAdapter {
    type Credential = ();

    fn configuration_identity(&self) -> &str {
        "worth-query-document-retention-operator-v1"
    }

    fn validate<'a>(
        &'a self,
        _credential: Self::Credential,
        _scope: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = worth_query_host::facade::runtime::time::system_time_sampler()
                .system_time()
                .unwrap();
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
    WorthQueryExternalPrincipalIdentity::new(OperatorIssuer, OPERATOR_SUBJECT)
        .expect("the operator external identity is valid")
}

/// Authenticates at the provider, but has no seeded application mapping or grant.
pub fn secondary_signer_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(OperatorIssuer, "secondary-signer").unwrap()
}

pub fn admit_operator_adapter<S: ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<S>,
    clock: worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock,
) -> WorthQueryAdmittedAuthenticationAdapter<S, OperatorAdapter> {
    admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(audience(), method(), clock),
        OperatorAdapter,
    )
    .expect("the operator adapter must be admitted")
}

pub fn authenticate_operator<S: ApplicationSchema>(
    schema: &WorthQueryInstalledApplicationSchema<S>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<S> {
    block_on(admit_operator_adapter(schema, scope.clock().clone()).authenticate((), scope))
        .expect("the operator must authenticate")
}

/// A bounded request scope: a hung step fails by deadline instead of waiting
/// forever. The bound is far beyond any court's runtime, so machine load
/// cannot expire an ordinary request; expiry courts build their own scope.
pub fn request_scope() -> WorthQueryRequestScope {
    let cancellation = WorthQueryCancellationSource::new();
    WorthQueryRequestScope::new(
        Some(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system().now() + Duration::from_secs(600)),
        cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
}

pub fn audience() -> WorthQueryAuthenticationAudience {
    WorthQueryAuthenticationAudience::of(DocumentRetentionHostAudience)
}

pub fn method() -> WorthQueryAuthenticationMethod {
    WorthQueryAuthenticationMethod::of(CertificationMethod)
}

pub fn block_on<F: Future>(future: F) -> F::Output {
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

#[path = "operator_identity/reviewer.rs"]
pub mod reviewer;
pub use reviewer::{
    authenticate_reviewer, authenticate_ungranted_reviewer, reviewer_identity,
    ungranted_reviewer_identity,
};
