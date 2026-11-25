//! Local authentication for checkpoint fixture requests.
use super::*;
use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, SystemTime};
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;

worth_query_decl::facade::worth_query_authentication_method!(LocalMethod);
worth_query_decl::facade::worth_query_authentication_audience!(CheckpointAudience);
worth_query_decl::facade::worth_query_authentication_audience!(NeutralHistoryAudience);
worth_query_decl::facade::worth_query_principal_issuer!(CheckpointInvalidLocalIssuer);
worth_query_decl::facade::worth_query_principal_issuer!(NeutralHistoryInvalidLocalIssuer);

#[derive(Clone, Copy)]
pub(in super::super) enum FixtureAuthenticationRealm {
    Checkpoint,
    NeutralHistory,
}

impl FixtureAuthenticationRealm {
    fn audience(self) -> authentication::WorthQueryAuthenticationAudience {
        match self {
            Self::Checkpoint => {
                authentication::WorthQueryAuthenticationAudience::of(CheckpointAudience)
            }
            Self::NeutralHistory => {
                authentication::WorthQueryAuthenticationAudience::of(NeutralHistoryAudience)
            }
        }
    }

    fn configuration_identity(self) -> &'static str {
        match self {
            Self::Checkpoint => "worth.query.certification.checkpoint-authentication.v1",
            Self::NeutralHistory => "worth.query.certification.neutral-history-authentication.v1",
        }
    }

    pub(in super::super) fn external_identity(self) -> WorthQueryExternalPrincipalIdentity {
        match self {
            Self::Checkpoint => WorthQueryExternalPrincipalIdentity::new(
                CheckpointInvalidLocalIssuer,
                "model-owner",
            ),
            Self::NeutralHistory => WorthQueryExternalPrincipalIdentity::new(
                NeutralHistoryInvalidLocalIssuer,
                "model-owner",
            ),
        }
        .unwrap()
    }
}

struct LocalIdentityAdapter(FixtureAuthenticationRealm);
struct LocalCredential;

impl authentication::WorthQueryAuthenticationAdapter for LocalIdentityAdapter {
    type Credential = LocalCredential;

    fn configuration_identity(&self) -> &str {
        self.0.configuration_identity()
    }

    fn validate<'a>(
        &'a self,
        _: Self::Credential,
        _: &'a authentication::WorthQueryRequestScope,
    ) -> authentication::WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            authentication::WorthQueryValidatedExternalPrincipal::new(
                self.0.external_identity(),
                self.0.audience(),
                authentication::WorthQueryAuthenticationMethod::of(LocalMethod),
                now,
                furthest_principal_expiry(now),
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

// Fixture law: expiry is not a fact here. Use the far end of SystemTime,
// rather than a timeout that a loaded count proof could outlive.
fn furthest_principal_expiry(now: SystemTime) -> SystemTime {
    let mut minimum = 0_u64;
    let mut maximum = u64::MAX;
    while minimum < maximum {
        let middle = minimum + (maximum - minimum).div_ceil(2);
        if now.checked_add(Duration::from_secs(middle)).is_some() {
            minimum = middle;
        } else {
            maximum = middle - 1;
        }
    }
    let end = now.checked_add(Duration::from_secs(minimum)).unwrap();
    let mut minimum = 0_u32;
    let mut maximum = 999_999_999;
    while minimum < maximum {
        let middle = minimum + (maximum - minimum).div_ceil(2);
        if end
            .checked_add(Duration::from_nanos(u64::from(middle)))
            .is_some()
        {
            minimum = middle;
        } else {
            maximum = middle - 1;
        }
    }
    end.checked_add(Duration::from_nanos(u64::from(minimum)))
        .unwrap()
}

pub(in super::super) fn external_identity() -> WorthQueryExternalPrincipalIdentity {
    FixtureAuthenticationRealm::Checkpoint.external_identity()
}

pub(in super::super) fn authenticate<Schema: ApplicationSchema + 'static>(
    application: &primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
) -> (
    authentication::WorthQueryRequestScope,
    authentication::WorthQueryAuthenticatedExternalPrincipal<Schema>,
) {
    authenticate_in_realm(
        application,
        FixtureAuthenticationRealm::Checkpoint,
        authentication::WorthQueryAdmissionClock::system(),
    )
}

pub(in super::super) fn authenticate_in_realm<Schema: ApplicationSchema + 'static>(
    application: &primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    realm: FixtureAuthenticationRealm,
    clock: authentication::WorthQueryAdmissionClock,
) -> (
    authentication::WorthQueryRequestScope,
    authentication::WorthQueryAuthenticatedExternalPrincipal<Schema>,
) {
    let cancellation = authentication::WorthQueryCancellationSource::new();
    // Fixture law: a request carries a deadline only when the deadline is the fact under test.
    let scope =
        authentication::WorthQueryRequestScope::new(None, cancellation.token(), clock.clone());
    let adapter = authentication::admit_authentication_adapter(
        application.installed_schema(),
        authentication::WorthQueryAuthenticationAdapterAdmission::new(
            realm.audience(),
            authentication::WorthQueryAuthenticationMethod::of(LocalMethod),
            clock,
        ),
        LocalIdentityAdapter(realm),
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

#[cfg(test)]
mod serial {
    use super::*;
    #[test]
    fn fixture_principal_expiry_cannot_be_a_wall_clock_timeout() {
        let cancellation = authentication::WorthQueryCancellationSource::new();
        let scope = authentication::WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
        let principal = block_on(authentication::WorthQueryAuthenticationAdapter::validate(
            &LocalIdentityAdapter(FixtureAuthenticationRealm::Checkpoint),
            LocalCredential,
            &scope,
        ))
        .unwrap();
        assert!(
            principal
                .expires_at()
                .checked_add(Duration::from_secs(1))
                .is_none(),
            "a count-proof principal must expire at the far end of SystemTime"
        );
    }
}
