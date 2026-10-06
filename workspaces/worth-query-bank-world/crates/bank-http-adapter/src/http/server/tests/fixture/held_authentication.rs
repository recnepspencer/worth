use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bank_domain::model::AccountId;
use bank_server::{BankAuthenticatedPrincipal, BankIdentityRuntime};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

use super::super::super::super::protocol::{BankHttpCredential, BankHttpDenial};
use super::super::super::authentication::BankHttpApplicationAuthenticator;
use super::{application, CausalHttpApplication};

pub(in crate::http::server::tests) struct HeldAuthenticationApplication {
    inner: CausalHttpApplication,
    calls: Arc<AtomicUsize>,
    release: Arc<tokio::sync::Semaphore>,
}

#[derive(Clone)]
pub(in crate::http::server::tests) struct HeldAuthenticationControl {
    calls: Arc<AtomicUsize>,
    release: Arc<tokio::sync::Semaphore>,
}

pub(in crate::http::server::tests) fn held_authentication_application(
    account: AccountId,
) -> (HeldAuthenticationApplication, HeldAuthenticationControl) {
    let calls = Arc::new(AtomicUsize::new(0));
    let release = Arc::new(tokio::sync::Semaphore::new(0));
    (
        HeldAuthenticationApplication {
            inner: application(account),
            calls: Arc::clone(&calls),
            release: Arc::clone(&release),
        },
        HeldAuthenticationControl { calls, release },
    )
}

impl HeldAuthenticationControl {
    pub(in crate::http::server::tests) async fn wait_for_calls(&self, expected: usize) {
        tokio::time::timeout(Duration::from_secs(2), async {
            while self.calls.load(Ordering::Acquire) < expected {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("held authentication should be reached");
    }

    pub(in crate::http::server::tests) fn release(&self, count: usize) {
        self.release.add_permits(count);
    }

    pub(in crate::http::server::tests) fn call_count(&self) -> usize {
        self.calls.load(Ordering::Acquire)
    }
}

impl super::super::super::inbound_completion::BankRailCompletionRuntime
    for HeldAuthenticationApplication
{
    fn runtime(&self) -> &BankIdentityRuntime {
        &self.inner.runtime
    }
}

impl BankHttpApplicationAuthenticator for HeldAuthenticationApplication {
    fn authenticate<'a>(
        &'a self,
        credential: BankHttpCredential,
        scope: &'a WorthQueryRequestScope,
    ) -> Pin<Box<dyn Future<Output = Result<BankAuthenticatedPrincipal, BankHttpDenial>> + Send + 'a>>
    {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::AcqRel);
            self.release
                .acquire()
                .await
                .expect("test authentication gate remains open")
                .forget();
            BankHttpApplicationAuthenticator::authenticate(&self.inner, credential, scope).await
        })
    }
}
