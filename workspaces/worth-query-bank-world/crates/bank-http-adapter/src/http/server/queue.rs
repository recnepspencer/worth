//! Bounded account-query dispatch. Owning money commands have their own persistent workers.
use super::super::protocol::{
    BankHttpAccountSummaryOutcome, BankHttpDenial, BankHttpDenialKind, BankHttpNextAction,
};
use super::application::{execute_account_summary, AdmittedAccountSummaryRequest};
use super::authentication::BankHttpApplicationAuthenticator;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Semaphore};
use tokio::task::{JoinHandle, JoinSet};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource;
#[derive(Clone)]
pub(super) struct BankHttpExecutionQueue {
    sender: mpsc::Sender<AccountSummaryJob>,
}
struct AccountSummaryJob {
    request: AdmittedAccountSummaryRequest,
    cancellation: WorthQueryCancellationSource,
    response: oneshot::Sender<BankHttpAccountSummaryOutcome>,
}
impl BankHttpExecutionQueue {
    pub(super) fn start<A: BankHttpApplicationAuthenticator>(
        application: Arc<A>,
        queue_capacity: usize,
        maximum_concurrency: usize,
    ) -> (Self, JoinHandle<()>) {
        let (sender, receiver) = mpsc::channel(queue_capacity);
        (
            Self { sender },
            tokio::spawn(dispatch(application, receiver, maximum_concurrency)),
        )
    }
    pub(super) async fn execute(
        &self,
        request: AdmittedAccountSummaryRequest,
    ) -> BankHttpAccountSummaryOutcome {
        let request_id = request.request_id.clone();
        let deadline = request.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let cancellation_guard = CancelOnDrop(cancellation.clone());
        let (response, receiver) = oneshot::channel();
        let response_wait = tokio::time::timeout_at(deadline.into(), receiver);
        tokio::pin!(response_wait);
        // Cancellation must precede disposal of the response future in every edition.
        let _guard = cancellation_guard;
        if self
            .sender
            .try_send(AccountSummaryJob {
                request,
                cancellation,
                response,
            })
            .is_err()
        {
            return denied(request_id, BankHttpDenialKind::Saturated);
        }
        match response_wait.await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(_)) => denied(request_id, BankHttpDenialKind::Unavailable),
            Err(_) => denied(request_id, BankHttpDenialKind::DeadlineExceeded),
        }
    }
}
struct CancelOnDrop(WorthQueryCancellationSource);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}
async fn dispatch<A: BankHttpApplicationAuthenticator>(
    application: Arc<A>,
    mut receiver: mpsc::Receiver<AccountSummaryJob>,
    maximum_concurrency: usize,
) {
    let concurrency = Arc::new(Semaphore::new(maximum_concurrency));
    let mut children = JoinSet::new();
    loop {
        while children.try_join_next().is_some() {}
        let Ok(permit) = Arc::clone(&concurrency).acquire_owned().await else {
            break;
        };
        let Some(job) = receiver.recv().await else {
            break;
        };
        let application = Arc::clone(&application);
        children.spawn(async move {
            let outcome = execute_account_summary(
                application.as_ref(),
                job.request,
                job.cancellation.token(),
            )
            .await;
            let _ = job.response.send(outcome);
            drop(permit);
        });
    }
    while children.join_next().await.is_some() {}
}
fn denied(request_id: String, kind: BankHttpDenialKind) -> BankHttpAccountSummaryOutcome {
    BankHttpAccountSummaryOutcome::Denied {
        request_id: Some(request_id),
        denial: BankHttpDenial::new(kind, BankHttpNextAction::Retry),
    }
}

#[cfg(test)]
#[path = "queue/cancellation_tests.rs"]
mod cancellation_tests;
