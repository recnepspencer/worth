//! The local principal reaches a Directory Home with no executor and refuses
//! unmapped, disabled and expired identities as an asynchronous adapter does.
use super::*;
use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::time::SystemTime;

worth_query_decl::facade::worth_query_authentication_audience!(HostAudience);
worth_query_decl::facade::worth_query_authentication_method!(TestMethod);

/// An ordinary asynchronous adapter vouching for one subject, standing in for
/// a remote identity provider.
struct RemoteDirectory {
    subject: &'static str,
    lifetime: Duration,
}

impl WorthQueryAuthenticationAdapter for RemoteDirectory {
    type Credential = ();
    fn configuration_identity(&self) -> &str {
        "worth.query.startup.remote.v1"
    }
    fn validate<'a>(
        &'a self,
        (): (),
        _: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                issuer(self.subject),
                WorthQueryAuthenticationAudience::of(HostAudience),
                WorthQueryAuthenticationMethod::of(TestMethod),
                now,
                now + self.lifetime,
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

/// Drives the remote adapter's future the way an executor would.
fn remote_principal(
    home: &Home,
    clock: &WorthQueryAdmissionClock,
    scope: &WorthQueryRequestScope,
    subject: &'static str,
    lifetime: Duration,
) -> WorthQueryAuthenticatedExternalPrincipal<Schema> {
    let adapter = admit_authentication_adapter(
        home.runtime().runtime().installed_schema(),
        admission(clock),
        RemoteDirectory { subject, lifetime },
    )
    .unwrap();
    let mut future = pin!(adapter.authenticate((), scope));
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(principal) => principal.unwrap(),
        Poll::Pending => panic!("the remote stand-in resolves on first poll"),
    }
}

#[test]
fn local_principal_reads_a_directory_home_and_refuses_like_a_remote_adapter() {
    // A Directory Home admits requests on the system monotonic clock.
    let clock = WorthQueryAdmissionClock::system();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open_home(
        &execution,
        WorthQueryApplicationLimits::ordinary_serial(clock.clone()),
    )
    .unwrap_or_else(|refused| panic!("ordinary serial Home opens: {refused:?}"));
    let scope = scope(&clock);
    let local = |subject, lifetime| {
        local_principal(&home, &clock, subject, lifetime)
            .authenticate_local(&scope)
            .expect("the local process authenticates on the calling thread")
    };
    let hour = Duration::from_secs(3600);

    let enabled = local("subject", hour);
    assert_eq!(read_balance(&home, &enabled, &scope), Ok(vec![7]));
    let mut refusals = Vec::new();
    for subject in ["unmapped", "disabled"] {
        let local_refusal = read_balance(&home, &local(subject, hour), &scope)
            .expect_err("an unresolvable identity is refused at request admission");
        let remote = remote_principal(&home, &clock, &scope, subject, hour);
        assert_eq!(
            read_balance(&home, &remote, &scope),
            Err(local_refusal.clone())
        );
        refusals.push(local_refusal);
    }
    assert_eq!(
        refusals,
        [
            "principal resolution Refused(UnknownPrincipal)",
            "principal resolution Refused(DisabledPrincipal)",
        ]
    );

    let brief = Duration::from_millis(50);
    let local_brief = local("subject", brief);
    let remote_brief = remote_principal(&home, &clock, &scope, "subject", brief);
    // The monotonic clock guarantees both finite lifetimes have elapsed.
    std::thread::sleep(brief + Duration::from_millis(20));
    assert!(local_brief.is_expired() && remote_brief.is_expired());
    let expired = read_balance(&home, &local_brief, &scope)
        .expect_err("an expired local proof is refused at request admission");
    assert_eq!(
        expired,
        "principal resolution Refused(ExpiredAuthentication)"
    );
    assert_eq!(read_balance(&home, &remote_brief, &scope), Err(expired));
    assert_eq!(read_balance(&home, &enabled, &scope), Ok(vec![7]));
    drop((enabled, local_brief, remote_brief));
    close_home(home, &execution);
}
