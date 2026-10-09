use super::*;

pub(super) fn external_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new("https://inbound-test.example", "alice").unwrap()
}

pub(super) fn authenticate(
    schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<
        InboundTestSchema,
    >,
    request: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<InboundTestSchema> {
    let adapter = admit_authentication_adapter(
        schema,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("inbound-test").unwrap(),
            WorthQueryAuthenticationMethod::new("test-identity").unwrap(),
        ),
        TestIdentityAdapter,
    )
    .unwrap();
    block_on(adapter.authenticate((), request)).unwrap()
}

struct TestIdentityAdapter;
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
                WorthQueryAuthenticationAudience::new("inbound-test").unwrap(),
                WorthQueryAuthenticationMethod::new("test-identity").unwrap(),
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
