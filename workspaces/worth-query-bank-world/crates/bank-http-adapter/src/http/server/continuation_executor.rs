use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Instant;

use bank_domain::model::AccountId;
use bank_server::{BankApplicationQueryDenial, BankReadControls};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationQueryResumeControls;

use super::super::protocol::{
    BankHttpAccountActivity, BankHttpAccountActivityPageOutcome, BankHttpCredential,
    BankHttpDenial, BankHttpDenialKind, BankHttpNextAction, BankHttpQueryCapabilityPurpose,
    BankHttpRequestControls,
};
use super::authenticated_owner::BankHttpAuthenticatedOwner;
use super::authentication::BankHttpApplicationAuthenticator;
use super::continuation_registry::{access, ResumeAdmission, SharedContinuationRegistry};
use super::query_denial::query_denial;
use super::query_publication::describe_query_publication;

mod activity;
pub(super) mod correction_history;
use activity::{execute_page, execute_resume};
mod replay;
#[cfg(test)]
pub(super) use replay::fault as replay_fault;
use replay::{fail_resume, revalidate_replay, revokes_cached_access};

pub(super) struct BankHttpContinuationExecutor {
    sender: mpsc::Sender<ContinuationCommand>,
}

pub(super) struct AdmittedPageRequest {
    pub(super) request_id: String,
    pub(super) credential: BankHttpCredential,
    pub(super) controls: BankHttpRequestControls,
    pub(super) account: AccountId,
    pub(super) deadline: Instant,
}

pub(super) struct AdmittedResumeRequest {
    pub(super) page: AdmittedPageRequest,
    pub(super) continuation: String,
}

enum ContinuationCommand {
    Page(ContinuationJob<AdmittedPageRequest>),
    Resume(ContinuationJob<AdmittedResumeRequest>),
    Correction(correction_history::HistoryJob),
}

struct ContinuationJob<R> {
    request: R,
    cancellation: WorthQueryCancellationSource,
    response: oneshot::Sender<BankHttpAccountActivityPageOutcome>,
}

impl Clone for BankHttpContinuationExecutor {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl BankHttpContinuationExecutor {
    pub(super) fn start<A>(
        application: Arc<A>,
        queue_capacity: usize,
        registry: SharedContinuationRegistry,
    ) -> (Self, JoinHandle<()>)
    where
        A: BankHttpApplicationAuthenticator,
    {
        let (sender, receiver) = mpsc::channel(queue_capacity);
        let task = tokio::spawn(run(application, receiver, registry));
        (Self { sender }, task)
    }

    pub(super) async fn page(
        &self,
        request: AdmittedPageRequest,
    ) -> BankHttpAccountActivityPageOutcome {
        let request_id = request.request_id.clone();
        let deadline = request.deadline;
        self.execute(ContinuationCommand::Page, request, request_id, deadline)
            .await
    }

    pub(super) async fn resume(
        &self,
        request: AdmittedResumeRequest,
    ) -> BankHttpAccountActivityPageOutcome {
        let request_id = request.page.request_id.clone();
        let deadline = request.page.deadline;
        self.execute(ContinuationCommand::Resume, request, request_id, deadline)
            .await
    }

    async fn execute<R>(
        &self,
        command: fn(ContinuationJob<R>) -> ContinuationCommand,
        request: R,
        request_id: String,
        deadline: Instant,
    ) -> BankHttpAccountActivityPageOutcome {
        let cancellation = WorthQueryCancellationSource::new();
        let (response, receiver) = oneshot::channel();
        let job = command(ContinuationJob {
            request,
            cancellation,
            response,
        });
        if self.sender.try_send(job).is_err() {
            return denied(Some(request_id), saturated());
        }

        match tokio::time::timeout_at(deadline.into(), receiver).await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(_)) => denied(Some(request_id), unavailable()),
            Err(_) => denied(Some(request_id), deadline_denial()),
        }
    }
}

async fn run<A>(
    application: Arc<A>,
    mut receiver: mpsc::Receiver<ContinuationCommand>,
    registry: SharedContinuationRegistry,
) where
    A: BankHttpApplicationAuthenticator,
{
    while let Some(command) = receiver.recv().await {
        match command {
            ContinuationCommand::Correction(job) => {
                let outcome =
                    correction_history::execute(application.as_ref(), &registry, &job).await;
                let _ = job.response.send(outcome);
            }
            ContinuationCommand::Page(job) => {
                let outcome = execute_page(application.as_ref(), &registry, &job).await;
                let _ = job.response.send(outcome);
            }
            ContinuationCommand::Resume(job) => {
                let outcome = execute_resume(application.as_ref(), &registry, &job).await;
                let _ = job.response.send(outcome);
            }
        }
    }
}

fn denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpAccountActivityPageOutcome {
    BankHttpAccountActivityPageOutcome::Denied { request_id, denial }
}

const fn malformed() -> BankHttpDenial {
    BankHttpDenial::new(
        BankHttpDenialKind::MalformedRequest,
        BankHttpNextAction::CorrectRequest,
    )
}

const fn saturated() -> BankHttpDenial {
    BankHttpDenial::new(BankHttpDenialKind::Saturated, BankHttpNextAction::Retry)
}

const fn unavailable() -> BankHttpDenial {
    BankHttpDenial::new(BankHttpDenialKind::Unavailable, BankHttpNextAction::Retry)
}

const fn deadline_denial() -> BankHttpDenial {
    BankHttpDenial::new(
        BankHttpDenialKind::DeadlineExceeded,
        BankHttpNextAction::Retry,
    )
}

const fn stale() -> BankHttpDenial {
    BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh)
}

const fn internal() -> BankHttpDenial {
    BankHttpDenial::new(BankHttpDenialKind::InternalDenied, BankHttpNextAction::None)
}
