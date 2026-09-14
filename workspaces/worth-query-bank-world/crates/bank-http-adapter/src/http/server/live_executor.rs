use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use bank_domain::model::AccountId;
use bank_server::BankAccountActivityLiveOutcome;
use tokio::sync::{mpsc, oneshot, OwnedSemaphorePermit, Semaphore};
use tokio_stream::Stream;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationLiveControls;

use super::super::protocol::{
    BankHttpAccountActivity, BankHttpAccountActivityEvent, BankHttpCredential, BankHttpDenial,
    BankHttpDenialKind, BankHttpNextAction, BankHttpQueryCapabilityPurpose,
    BankHttpRequestControls,
};
use super::authentication::BankHttpApplicationAuthenticator;
use super::query_denial::query_denial;
use super::query_publication::describe_query_publication;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub(super) struct BankHttpLiveExecutor {
    sender: mpsc::Sender<OpenAccountActivityStream>,
    event_capacity: usize,
    active_streams: Arc<Semaphore>,
}

pub(super) struct AdmittedAccountActivityStreamRequest {
    pub(super) request_id: String,
    pub(super) credential: BankHttpCredential,
    pub(super) controls: BankHttpRequestControls,
    pub(super) account: AccountId,
    pub(super) source_buffer_capacity: usize,
    pub(super) deadline: Instant,
}

struct OpenAccountActivityStream {
    request: AdmittedAccountActivityStreamRequest,
    events: mpsc::Sender<BankHttpAccountActivityEvent>,
    terminal: Option<oneshot::Sender<BankHttpAccountActivityEvent>>,
    _active_stream: OwnedSemaphorePermit,
}

pub(super) struct BankHttpLiveEventStream {
    events: mpsc::Receiver<BankHttpAccountActivityEvent>,
    terminal: oneshot::Receiver<BankHttpAccountActivityEvent>,
    events_closed: bool,
    terminal_closed: bool,
}

impl Stream for BankHttpLiveEventStream {
    type Item = BankHttpAccountActivityEvent;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.events_closed {
            match self.events.poll_recv(context) {
                Poll::Ready(Some(event)) => return Poll::Ready(Some(event)),
                Poll::Ready(None) => self.events_closed = true,
                Poll::Pending => return Poll::Pending,
            }
        }
        if self.terminal_closed {
            return Poll::Ready(None);
        }
        match Pin::new(&mut self.terminal).poll(context) {
            Poll::Ready(result) => {
                self.terminal_closed = true;
                Poll::Ready(result.ok())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl BankHttpLiveExecutor {
    pub(super) fn start<A>(
        application: Arc<A>,
        open_queue_capacity: usize,
        event_capacity: usize,
        maximum_active_streams: usize,
    ) -> io::Result<(Self, JoinHandle<io::Result<()>>)>
    where
        A: BankHttpApplicationAuthenticator,
    {
        let (sender, receiver) = mpsc::channel(open_queue_capacity);
        let thread = std::thread::Builder::new()
            .name("bank-http-live-executor".to_owned())
            .spawn(move || run_local_executor(application, receiver))?;
        Ok((
            Self {
                sender,
                event_capacity,
                active_streams: Arc::new(Semaphore::new(maximum_active_streams)),
            },
            thread,
        ))
    }

    pub(super) fn open(
        &self,
        request: AdmittedAccountActivityStreamRequest,
    ) -> Result<BankHttpLiveEventStream, BankHttpDenial> {
        let active_stream = Arc::clone(&self.active_streams)
            .try_acquire_owned()
            .map_err(|_| {
                BankHttpDenial::new(BankHttpDenialKind::Saturated, BankHttpNextAction::Retry)
            })?;
        let (events, receiver) = mpsc::channel(self.event_capacity);
        let (terminal, terminal_receiver) = oneshot::channel();
        self.sender
            .try_send(OpenAccountActivityStream {
                request,
                events,
                terminal: Some(terminal),
                _active_stream: active_stream,
            })
            .map_err(|_| {
                BankHttpDenial::new(BankHttpDenialKind::Saturated, BankHttpNextAction::Retry)
            })?;
        Ok(BankHttpLiveEventStream {
            events: receiver,
            terminal: terminal_receiver,
            events_closed: false,
            terminal_closed: false,
        })
    }
}

fn run_local_executor<A>(
    application: Arc<A>,
    receiver: mpsc::Receiver<OpenAccountActivityStream>,
) -> io::Result<()>
where
    A: BankHttpApplicationAuthenticator,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let local = tokio::task::LocalSet::new();
    local.block_on(&runtime, dispatch_streams(application, receiver));
    Ok(())
}

async fn dispatch_streams<A>(
    application: Arc<A>,
    mut receiver: mpsc::Receiver<OpenAccountActivityStream>,
) where
    A: BankHttpApplicationAuthenticator,
{
    while let Some(stream) = receiver.recv().await {
        let application = Arc::clone(&application);
        tokio::task::spawn_local(run_stream(application, stream));
    }
}

async fn run_stream<A>(application: Arc<A>, stream: OpenAccountActivityStream)
where
    A: BankHttpApplicationAuthenticator,
{
    let OpenAccountActivityStream {
        request,
        events,
        mut terminal,
        _active_stream,
    } = stream;
    let request_id = request.request_id;
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(request.deadline, cancellation.token());
    let principal = match application.authenticate(request.credential, &scope).await {
        Ok(principal) => principal,
        Err(denial) => {
            send_terminal(&mut terminal, denied(request_id, denial));
            return;
        }
    };
    let controls = match WorthQueryApplicationLiveControls::bounded(
        scope,
        request.source_buffer_capacity,
        request.controls.maximum_results,
        request.controls.maximum_work,
    ) {
        Ok(controls) => controls,
        Err(_) => {
            send_terminal(&mut terminal, malformed(request_id));
            return;
        }
    };
    let mut lease = match application
        .runtime()
        .account_activity(request.account)
        .as_principal(&principal)
        .subscribe(controls)
    {
        Ok(lease) => lease,
        Err(denial) => {
            send_terminal(&mut terminal, denied(request_id, query_denial(denial)));
            return;
        }
    };
    if events
        .try_send(BankHttpAccountActivityEvent::Opened {
            request_id: request_id.clone(),
        })
        .is_err()
    {
        cancellation.cancel();
        let _ = lease.close();
        return;
    }
    let mut poll = tokio::time::interval(Duration::from_millis(20));
    let authentication_expiry =
        tokio::time::sleep_until(principal.authentication_valid_until().into());
    tokio::pin!(authentication_expiry);
    loop {
        tokio::select! {
            _ = events.closed() => {
                cancellation.cancel();
                let _ = lease.close();
                return;
            }
            _ = &mut authentication_expiry => {
                let _ = lease.close();
                send_terminal(
                    &mut terminal,
                    denied(
                        request_id,
                        BankHttpDenial::new(
                            BankHttpDenialKind::Unauthenticated,
                            BankHttpNextAction::Authenticate,
                        ),
                    ),
                );
                return;
            }
            _ = poll.tick() => {
                let delivery_scope = WorthQueryRequestScope::new(
                    request.deadline,
                    cancellation.token(),
                );
                if let Some(event) = live_event(
                    &request_id,
                    lease.poll(&principal, &delivery_scope),
                ) {
                    if !send_live_event(&events, &mut terminal, event, &request_id) {
                        cancellation.cancel();
                        let _ = lease.close();
                        return;
                    }
                }
            }
        }
    }
}

fn send_live_event(
    events: &mpsc::Sender<BankHttpAccountActivityEvent>,
    terminal: &mut Option<oneshot::Sender<BankHttpAccountActivityEvent>>,
    event: BankHttpAccountActivityEvent,
    request_id: &str,
) -> bool {
    if is_terminal(&event) {
        send_terminal(terminal, event);
        return false;
    }
    match events.try_send(event) {
        Ok(()) => true,
        Err(mpsc::error::TrySendError::Full(_)) => {
            send_terminal(
                terminal,
                BankHttpAccountActivityEvent::Overflow {
                    request_id: request_id.to_owned(),
                    missed_commit_batches: 1,
                },
            );
            false
        }
        Err(mpsc::error::TrySendError::Closed(_)) => false,
    }
}

fn live_event(
    request_id: &str,
    outcome: BankAccountActivityLiveOutcome,
) -> Option<BankHttpAccountActivityEvent> {
    let request_id = request_id.to_owned();
    match outcome {
        BankAccountActivityLiveOutcome::Delivered(update) => {
            Some(BankHttpAccountActivityEvent::Update {
                request_id,
                activity: BankHttpAccountActivity::from(update.result()),
                publication: describe_query_publication(
                    update.receipt(),
                    BankHttpQueryCapabilityPurpose::AccountActivityReview,
                ),
            })
        }
        BankAccountActivityLiveOutcome::Pending => None,
        BankAccountActivityLiveOutcome::Overflow(overflow) => {
            Some(BankHttpAccountActivityEvent::Overflow {
                request_id,
                missed_commit_batches: overflow.missed_commit_batches(),
            })
        }
        BankAccountActivityLiveOutcome::AuthorizationDenied(_)
        | BankAccountActivityLiveOutcome::ProjectionDenied(_)
        | BankAccountActivityLiveOutcome::CauseDenied(_) => Some(denied(
            request_id,
            BankHttpDenial::new(
                BankHttpDenialKind::PermissionDenied,
                BankHttpNextAction::None,
            ),
        )),
        BankAccountActivityLiveOutcome::StalePrincipal
        | BankAccountActivityLiveOutcome::StaleScope => Some(denied(
            request_id,
            BankHttpDenial::new(BankHttpDenialKind::Stale, BankHttpNextAction::Refresh),
        )),
        BankAccountActivityLiveOutcome::Cancelled => {
            Some(BankHttpAccountActivityEvent::Cancelled { request_id })
        }
        BankAccountActivityLiveOutcome::DeadlineExceeded => {
            Some(BankHttpAccountActivityEvent::DeadlineExceeded { request_id })
        }
        BankAccountActivityLiveOutcome::Closed => {
            Some(BankHttpAccountActivityEvent::Closed { request_id })
        }
        BankAccountActivityLiveOutcome::Unavailable => {
            Some(BankHttpAccountActivityEvent::Unavailable { request_id })
        }
    }
}

const fn is_terminal(event: &BankHttpAccountActivityEvent) -> bool {
    !matches!(
        event,
        BankHttpAccountActivityEvent::Opened { .. } | BankHttpAccountActivityEvent::Update { .. }
    )
}

fn denied(request_id: String, denial: BankHttpDenial) -> BankHttpAccountActivityEvent {
    BankHttpAccountActivityEvent::Denied { request_id, denial }
}

fn malformed(request_id: String) -> BankHttpAccountActivityEvent {
    denied(
        request_id,
        BankHttpDenial::new(
            BankHttpDenialKind::MalformedRequest,
            BankHttpNextAction::CorrectRequest,
        ),
    )
}

fn send_terminal(
    terminal: &mut Option<oneshot::Sender<BankHttpAccountActivityEvent>>,
    event: BankHttpAccountActivityEvent,
) {
    if let Some(terminal) = terminal.take() {
        let _ = terminal.send(event);
    }
}
