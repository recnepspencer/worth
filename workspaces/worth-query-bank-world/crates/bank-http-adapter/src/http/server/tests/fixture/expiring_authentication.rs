//! Authentication that, when armed, admits the principal and then outlives the
//! request deadline, so the next deadline check the request reaches is the
//! adapter's own read after authentication.
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bank_domain::model::AccountId;
use bank_server::{BankAuthenticatedPrincipal, BankIdentityRuntime};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

use super::super::super::super::protocol::{BankHttpCredential, BankHttpDenial};
use super::super::super::authentication::BankHttpApplicationAuthenticator;
use super::{application, CausalHttpApplication};

pub(in crate::http::server::tests) struct ExpiringAuthenticationApplication {
    inner: CausalHttpApplication,
    armed: Arc<AtomicBool>,
}

#[derive(Clone)]
pub(in crate::http::server::tests) struct ExpiringAuthenticationControl {
    armed: Arc<AtomicBool>,
}

pub(in crate::http::server::tests) fn expiring_authentication_application(
    account: AccountId,
) -> (
    ExpiringAuthenticationApplication,
    ExpiringAuthenticationControl,
) {
    let armed = Arc::new(AtomicBool::new(false));
    (
        ExpiringAuthenticationApplication {
            inner: application(account),
            armed: Arc::clone(&armed),
        },
        ExpiringAuthenticationControl { armed },
    )
}

impl ExpiringAuthenticationControl {
    pub(in crate::http::server::tests) fn arm(&self, armed: bool) {
        self.armed.store(armed, Ordering::Release);
    }
}

impl super::super::super::inbound_completion::BankRailCompletionRuntime
    for ExpiringAuthenticationApplication
{
    fn runtime(&self) -> &BankIdentityRuntime {
        &self.inner.runtime
    }
}

impl BankHttpApplicationAuthenticator for ExpiringAuthenticationApplication {
    fn authenticate<'a>(
        &'a self,
        credential: BankHttpCredential,
        scope: &'a WorthQueryRequestScope,
    ) -> Pin<Box<dyn Future<Output = Result<BankAuthenticatedPrincipal, BankHttpDenial>> + Send + 'a>>
    {
        Box::pin(async move {
            let principal =
                BankHttpApplicationAuthenticator::authenticate(&self.inner, credential, scope)
                    .await?;
            if self.armed.load(Ordering::Acquire) {
                let deadline = scope
                    .deadline()
                    .expect("an armed expiring request carries a deadline");
                tokio::time::sleep_until((deadline + Duration::from_millis(1)).into()).await;
                assert!(scope.interruption().is_some(), "the deadline has elapsed");
            }
            Ok(principal)
        })
    }
}
