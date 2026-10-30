//! The host admits two distinct credentials against actual installed principals.
use super::schema::Schema;
use std::time::{Duration, SystemTime};
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    declaration::authentication::WorthQueryExternalPrincipalIdentity,
    domain::WorthQueryInstalledApplicationSchema,
};

use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::time::Instant;

pub fn request_scope() -> WorthQueryRequestScope {
    let cancellation = WorthQueryCancellationSource::new();
    WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(120)),
        cancellation.token(),
    )
}

fn block_on<T>(future: impl Future<Output = T>) -> T {
    let mut future = pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("the local credential adapter completes synchronously"),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Actor {
    Author,
    Reviewer,
}
impl Actor {
    pub fn subject(self) -> &'static str {
        match self {
            Self::Author => "author",
            Self::Reviewer => "reviewer",
        }
    }
    pub fn identity(self) -> WorthQueryExternalPrincipalIdentity {
        WorthQueryExternalPrincipalIdentity::new("https://proposals.invalid", self.subject())
            .unwrap()
    }
}

struct Adapter;
impl WorthQueryAuthenticationAdapter for Adapter {
    type Credential = Actor;
    fn configuration_identity(&self) -> &str {
        "proposal-application.authentication.v1"
    }
    fn validate<'a>(
        &'a self,
        actor: Actor,
        _: &'a WorthQueryRequestScope,
    ) -> WorthQueryAuthenticationFuture<'a> {
        Box::pin(async move {
            let now = SystemTime::now();
            WorthQueryValidatedExternalPrincipal::new(
                actor.identity(),
                WorthQueryAuthenticationAudience::new("proposals").unwrap(),
                WorthQueryAuthenticationMethod::new("local-credential").unwrap(),
                now,
                now + Duration::from_secs(600),
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

pub struct Caller<'scope> {
    pub(super) principal: WorthQueryAuthenticatedExternalPrincipal<Schema>,
    pub(super) scope: &'scope WorthQueryRequestScope,
}

pub fn authenticate<'scope>(
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
    actor: Actor,
    scope: &'scope WorthQueryRequestScope,
) -> Result<Caller<'scope>, WorthQueryAuthenticationDenial> {
    let adapter = admit_authentication_adapter(
        installed,
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("proposals").unwrap(),
            WorthQueryAuthenticationMethod::new("local-credential").unwrap(),
        ),
        Adapter,
    )
    .unwrap();
    block_on(adapter.authenticate(actor, scope)).map(|principal| Caller { principal, scope })
}
