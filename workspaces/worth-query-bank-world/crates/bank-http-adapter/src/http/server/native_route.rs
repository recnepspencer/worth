//! Route execution enters the same bounded, thread-affine native custody worker.
use super::super::protocol::BankHttpDenialKind;
use super::authenticated_owner::BankHttpAuthenticatedOwner;
use bank_server::BankManagedCleanup;
use std::{future::Future, pin::Pin};
use tokio::sync::oneshot;

pub(super) enum NativeRouteResult<T> {
    Applied(T),
    Estate {
        record: super::estate_source_custody::EstateSourceRecord,
        outcome: T,
    },
    Cleanup {
        owner: BankHttpAuthenticatedOwner,
        custody: BankManagedCleanup,
        outcome: T,
    },
}
impl<T> From<T> for NativeRouteResult<T> {
    fn from(value: T) -> Self {
        Self::Applied(value)
    }
}
pub(super) struct NativeReply {
    pub(super) cleanup: Option<(BankHttpAuthenticatedOwner, BankManagedCleanup)>,
    pub(super) estate: Option<super::estate_source_custody::EstateSourceRecord>,
    pub(super) reply: Box<dyn FnOnce() + Send>,
}
pub(super) trait NativeJob: Send {
    fn run(self: Box<Self>) -> Pin<Box<dyn Future<Output = NativeReply>>>;
    fn reject(self: Box<Self>, kind: BankHttpDenialKind);
}
struct RouteJob<T, F, D> {
    work: F,
    denied: D,
    response: oneshot::Sender<T>,
}
pub(super) fn job<T, F, Fut, D>(
    work: F,
    denied: D,
    response: oneshot::Sender<T>,
) -> Box<dyn NativeJob>
where
    T: Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = NativeRouteResult<T>> + 'static,
    D: FnOnce(BankHttpDenialKind) -> T + Send + 'static,
{
    Box::new(RouteJob {
        work,
        denied,
        response,
    })
}
impl<T, F, Fut, D> NativeJob for RouteJob<T, F, D>
where
    T: Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = NativeRouteResult<T>> + 'static,
    D: FnOnce(BankHttpDenialKind) -> T + Send + 'static,
{
    fn run(self: Box<Self>) -> Pin<Box<dyn Future<Output = NativeReply>>> {
        Box::pin(async move {
            let Self { work, response, .. } = *self;
            let (custody, estate, outcome) = match work().await {
                NativeRouteResult::Applied(outcome) => (None, None, outcome),
                NativeRouteResult::Cleanup {
                    owner,
                    custody,
                    outcome,
                } => (Some((owner, custody)), None, outcome),
                NativeRouteResult::Estate { record, outcome } => (None, Some(record), outcome),
            };
            NativeReply {
                cleanup: custody,
                estate,
                reply: Box::new(move || {
                    let _ = response.send(outcome);
                }),
            }
        })
    }
    fn reject(self: Box<Self>, kind: BankHttpDenialKind) {
        let Self {
            denied, response, ..
        } = *self;
        let _ = response.send(denied(kind));
    }
}
