use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

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
use super::continuation_registry::{BankHttpContinuationRegistry, ResumeAdmission};
use super::query_denial::query_denial;
use super::query_publication::describe_query_publication;

mod replay;
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
        registry_capacity: usize,
        lifetime: Duration,
    ) -> (Self, JoinHandle<()>)
    where
        A: BankHttpApplicationAuthenticator,
    {
        let (sender, receiver) = mpsc::channel(queue_capacity);
        let task = tokio::spawn(run(
            application,
            receiver,
            BankHttpContinuationRegistry::new(registry_capacity, lifetime),
        ));
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
        let outcome = match tokio::time::timeout_at(deadline.into(), receiver).await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(_)) => denied(Some(request_id), unavailable()),
            Err(_) => denied(Some(request_id), deadline_denial()),
        };
        outcome
    }
}

async fn run<A>(
    application: Arc<A>,
    mut receiver: mpsc::Receiver<ContinuationCommand>,
    mut registry: BankHttpContinuationRegistry,
) where
    A: BankHttpApplicationAuthenticator,
{
    while let Some(command) = receiver.recv().await {
        match command {
            ContinuationCommand::Page(job) => {
                let outcome = execute_page(application.as_ref(), &mut registry, &job).await;
                let _ = job.response.send(outcome);
            }
            ContinuationCommand::Resume(job) => {
                let outcome = execute_resume(application.as_ref(), &mut registry, &job).await;
                let _ = job.response.send(outcome);
            }
        }
    }
}

async fn execute_page<A>(
    application: &A,
    registry: &mut BankHttpContinuationRegistry,
    job: &ContinuationJob<AdmittedPageRequest>,
) -> BankHttpAccountActivityPageOutcome
where
    A: BankHttpApplicationAuthenticator,
{
    let request = &job.request;
    let scope = WorthQueryRequestScope::new(request.deadline, job.cancellation.token());
    let principal = match application
        .authenticate(request.credential.clone(), &scope)
        .await
    {
        Ok(principal) => principal,
        Err(denial) => return denied(Some(request.request_id.clone()), denial),
    };
    let owner = BankHttpAuthenticatedOwner::from_principal(&principal);
    if let Some((token, outcome)) =
        registry.replay_initial(&owner, request.account, &request.request_id)
    {
        if matches!(
            outcome,
            BankHttpAccountActivityPageOutcome::Delivered { .. }
        ) {
            if let Err(denial) = revalidate_replay(
                application,
                &principal,
                request.account,
                &request.controls,
                &scope,
            ) {
                if revokes_cached_access(denial) {
                    registry.deny_protected_replay(&token, denial);
                }
                return denied(Some(request.request_id.clone()), denial);
            }
        }
        return outcome;
    }
    let controls = match page_controls(&request.controls, &scope) {
        Some(controls) => controls,
        None => return denied(Some(request.request_id.clone()), malformed()),
    };
    let page = match application
        .runtime()
        .account_activity(request.account)
        .as_principal(&principal)
        .page(controls)
    {
        Ok(page) => page,
        Err(error) => return denied(Some(request.request_id.clone()), query_denial(error)),
    };
    let (published, continuation) = page.into_parts();
    let Some(activity) = published.rows().first().map(BankHttpAccountActivity::from) else {
        return denied(Some(request.request_id.clone()), internal());
    };
    let publication = describe_query_publication(
        published.receipt(),
        BankHttpQueryCapabilityPurpose::AccountActivityReview,
    );
    registry
        .register_initial(
            owner,
            request.account,
            request.request_id.clone(),
            continuation,
            activity,
            publication,
        )
        .unwrap_or_else(|_| denied(Some(request.request_id.clone()), saturated()))
}

async fn execute_resume<A>(
    application: &A,
    registry: &mut BankHttpContinuationRegistry,
    job: &ContinuationJob<AdmittedResumeRequest>,
) -> BankHttpAccountActivityPageOutcome
where
    A: BankHttpApplicationAuthenticator,
{
    let request = &job.request.page;
    let scope = WorthQueryRequestScope::new(request.deadline, job.cancellation.token());
    let principal = match application
        .authenticate(request.credential.clone(), &scope)
        .await
    {
        Ok(principal) => principal,
        Err(denial) => return denied(Some(request.request_id.clone()), denial),
    };
    let owner = BankHttpAuthenticatedOwner::from_principal(&principal);
    let controls = match resume_controls(&request.controls, &scope) {
        Some(controls) => controls,
        None => return denied(Some(request.request_id.clone()), malformed()),
    };
    if let Some(interruption) = scope.interruption() {
        let denial = match interruption {
            WorthQueryRequestInterruption::Cancelled => {
                BankHttpDenial::new(BankHttpDenialKind::Cancelled, BankHttpNextAction::Retry)
            }
            WorthQueryRequestInterruption::DeadlineExceeded => deadline_denial(),
        };
        return denied(Some(request.request_id.clone()), denial);
    }
    let continuation = match registry.begin_resume(
        &owner,
        request.account,
        &request.request_id,
        &job.request.continuation,
    ) {
        ResumeAdmission::Execute(continuation) => continuation,
        ResumeAdmission::Replay(outcome) => {
            if matches!(
                outcome,
                BankHttpAccountActivityPageOutcome::Delivered { .. }
            ) {
                if let Err(denial) = revalidate_replay(
                    application,
                    &principal,
                    request.account,
                    &request.controls,
                    &scope,
                ) {
                    if revokes_cached_access(denial) {
                        registry.deny_protected_replay(&job.request.continuation, denial);
                    }
                    return denied(Some(request.request_id.clone()), denial);
                }
            }
            return outcome;
        }
        ResumeAdmission::InFlight => return denied(Some(request.request_id.clone()), saturated()),
        ResumeAdmission::Unavailable => return denied(Some(request.request_id.clone()), stale()),
    };
    let page = application
        .runtime()
        .account_activity(request.account)
        .as_principal(&principal)
        .resume(continuation, controls);
    complete_resume(registry, &job.request.continuation, request, page)
}

fn complete_resume(
    registry: &mut BankHttpContinuationRegistry,
    token: &str,
    request: &AdmittedPageRequest,
    page: Result<bank_server::BankAccountActivityPageResult, BankApplicationQueryDenial>,
) -> BankHttpAccountActivityPageOutcome {
    match page {
        Ok(page) => {
            let (published, continuation) = page.into_parts();
            let Some(activity) = published.rows().first().map(BankHttpAccountActivity::from) else {
                return fail_resume(registry, token, request, internal());
            };
            let publication = describe_query_publication(
                published.receipt(),
                BankHttpQueryCapabilityPurpose::AccountActivityReview,
            );
            registry.complete_resume(
                token,
                request.request_id.clone(),
                activity,
                continuation,
                publication,
            )
        }
        Err(error) => fail_resume(registry, token, request, query_denial(error)),
    }
}

fn page_controls(
    controls: &BankHttpRequestControls,
    scope: &WorthQueryRequestScope,
) -> Option<BankReadControls> {
    BankReadControls::current(
        scope.clone(),
        controls.maximum_results,
        controls.maximum_work,
    )
    .ok()
}

fn resume_controls<'a>(
    controls: &BankHttpRequestControls,
    scope: &'a WorthQueryRequestScope,
) -> Option<WorthQueryApplicationQueryResumeControls<'a>> {
    Some(WorthQueryApplicationQueryResumeControls::new(
        NonZeroUsize::new(controls.maximum_results)?,
        NonZeroUsize::new(controls.maximum_work)?,
        scope,
    ))
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
