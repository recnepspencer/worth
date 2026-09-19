use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, Semaphore};
use tokio::task::JoinHandle;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource;

use super::super::protocol::{
    BankHttpAccountSummaryOutcome, BankHttpDenial, BankHttpDenialKind, BankHttpMutationFailureKind,
    BankHttpMutationOutcome, BankHttpNextAction,
};
use super::application::{execute_account_summary, AdmittedAccountSummaryRequest};
use super::authentication::BankHttpApplicationAuthenticator;
use super::mutation_application::{execute_mutation, AdmittedBankHttpMutationRequest};

pub(super) struct BankHttpExecutionQueue {
    sender: mpsc::Sender<BankHttpJob>,
}

enum BankHttpJob {
    AccountSummary(BankHttpAccountSummaryJob),
    Mutation(BankHttpMutationJob),
}

struct BankHttpAccountSummaryJob {
    request: AdmittedAccountSummaryRequest,
    cancellation: WorthQueryCancellationSource,
    response: oneshot::Sender<BankHttpAccountSummaryOutcome>,
}

struct BankHttpMutationJob {
    request: AdmittedBankHttpMutationRequest,
    cancellation: WorthQueryCancellationSource,
    response: oneshot::Sender<BankHttpMutationOutcome>,
}

impl Clone for BankHttpExecutionQueue {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl BankHttpExecutionQueue {
    pub(super) fn start<A>(
        application: Arc<A>,
        queue_capacity: usize,
        maximum_concurrency: usize,
    ) -> (Self, JoinHandle<()>)
    where
        A: BankHttpApplicationAuthenticator,
    {
        let (sender, receiver) = mpsc::channel(queue_capacity);
        let dispatcher = tokio::spawn(dispatch(application, receiver, maximum_concurrency));
        (Self { sender }, dispatcher)
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
        let job = BankHttpJob::AccountSummary(BankHttpAccountSummaryJob {
            request,
            cancellation,
            response,
        });
        if self.sender.try_send(job).is_err() {
            return denied(request_id, BankHttpDenialKind::Saturated);
        }
        let outcome = match tokio::time::timeout_at(deadline.into(), receiver).await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(_)) => denied(request_id, BankHttpDenialKind::Unavailable),
            Err(_) => denied(request_id, BankHttpDenialKind::DeadlineExceeded),
        };
        drop(cancellation_guard);
        outcome
    }

    pub(super) async fn execute_mutation(
        &self,
        request: AdmittedBankHttpMutationRequest,
    ) -> BankHttpMutationOutcome {
        let request_id = request.request_id.clone();
        let deadline = request.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let cancellation_guard = CancelOnDrop(cancellation.clone());
        let (response, receiver) = oneshot::channel();
        let job = BankHttpJob::Mutation(BankHttpMutationJob {
            request,
            cancellation,
            response,
        });
        if self.sender.try_send(job).is_err() {
            return mutation_denied(request_id, BankHttpDenialKind::Saturated);
        }
        let outcome = match tokio::time::timeout_at(deadline.into(), receiver).await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(_)) => mutation_denied(request_id, BankHttpDenialKind::Unavailable),
            Err(_) => mutation_denied(request_id, BankHttpDenialKind::DeadlineExceeded),
        };
        drop(cancellation_guard);
        outcome
    }
}

struct CancelOnDrop(WorthQueryCancellationSource);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

async fn dispatch<A>(
    application: Arc<A>,
    mut receiver: mpsc::Receiver<BankHttpJob>,
    maximum_concurrency: usize,
) where
    A: BankHttpApplicationAuthenticator,
{
    let concurrency = Arc::new(Semaphore::new(maximum_concurrency));
    loop {
        let Ok(permit) = Arc::clone(&concurrency).acquire_owned().await else {
            return;
        };
        let Some(job) = receiver.recv().await else {
            return;
        };
        let application = Arc::clone(&application);
        tokio::spawn(async move {
            match job {
                BankHttpJob::AccountSummary(job) => {
                    let outcome = execute_account_summary(
                        application.as_ref(),
                        job.request,
                        job.cancellation.token(),
                    )
                    .await;
                    let _ = job.response.send(outcome);
                }
                BankHttpJob::Mutation(job) => {
                    let outcome = execute_mutation(
                        application.as_ref(),
                        job.request,
                        job.cancellation.token(),
                    )
                    .await;
                    let _ = job.response.send(outcome);
                }
            }
            drop(permit);
        });
    }
}

fn mutation_denied(request_id: String, kind: BankHttpDenialKind) -> BankHttpMutationOutcome {
    let failure = match kind {
        BankHttpDenialKind::DeadlineExceeded => BankHttpMutationFailureKind::DeadlineExceeded,
        _ => BankHttpMutationFailureKind::Aborted,
    };
    BankHttpMutationOutcome::NotApplied {
        request_id: Some(request_id),
        failure,
        stale_fact_count: None,
        provider_recovery: None,
        denial: BankHttpDenial::new(kind, BankHttpNextAction::Retry),
    }
}

fn denied(request_id: String, kind: BankHttpDenialKind) -> BankHttpAccountSummaryOutcome {
    let next_action = match kind {
        BankHttpDenialKind::Saturated
        | BankHttpDenialKind::Unavailable
        | BankHttpDenialKind::DeadlineExceeded => BankHttpNextAction::Retry,
        _ => BankHttpNextAction::None,
    };
    BankHttpAccountSummaryOutcome::Denied {
        request_id: Some(request_id),
        denial: BankHttpDenial::new(kind, next_action),
    }
}
