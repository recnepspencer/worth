use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::*;

worth_query_declaration::worth_query_authentication_audience!(InboundTestAudience);
worth_query_declaration::worth_query_principal_issuer!(InboundTestExampleIssuer);
worth_query_declaration::worth_query_authentication_method!(TestIdentityMethod);

pub fn external_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new(InboundTestExampleIssuer, "alice").unwrap()
}

pub fn authenticate(
    schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<
        InboundTestSchema,
    >,
    request: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<InboundTestSchema> {
    let adapter = admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::of(InboundTestAudience),
            WorthQueryAuthenticationMethod::of(TestIdentityMethod), worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()),
        TestIdentityAdapter,
    )
    .unwrap();
    block_on(adapter.authenticate((), request)).unwrap()
}

pub struct TestIdentityAdapter;
impl WorthQueryAuthenticationAdapter for TestIdentityAdapter {
    type Credential = ();
    fn configuration_identity(&self) -> &str {
        "inbound-test-identity-adapter"
    }
    fn validate<'a>(
        &'a self,
        _: (),
        _: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                external_identity(),
                WorthQueryAuthenticationAudience::of(InboundTestAudience),
                WorthQueryAuthenticationMethod::of(TestIdentityMethod),
                now,
                now + Duration::from_secs(60),
                vec![],
            )
            .map_err(|_| {
                WorthQueryAuthenticationAdapterFailure::new(
                    WorthQueryAuthenticationAdapterFailureKind::ProtocolViolation,
                )
            })
        })
    }
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
