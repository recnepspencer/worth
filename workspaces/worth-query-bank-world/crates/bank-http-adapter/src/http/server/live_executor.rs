use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use bank_domain::model::AccountId;
use bank_server::{BankAccountActivityLiveLease, BankApplicationLiveCloseOutcome};
use tokio::sync::{mpsc, oneshot, watch, OwnedSemaphorePermit, Semaphore};
use tokio::task::JoinSet;
use tokio_stream::Stream;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationLiveControls;

use super::super::protocol::{
    BankHttpAccountActivityEvent, BankHttpCredential, BankHttpDenial, BankHttpDenialKind,
    BankHttpNextAction, BankHttpRequestControls,
};
use super::authentication::BankHttpApplicationAuthenticator;
use super::query_denial::query_denial;

mod events;
use events::{denied, live_event, malformed, send_live_event, send_terminal};

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub(super) struct BankHttpLiveExecutor {
    sender: mpsc::Sender<OpenAccountActivityStream>,
    shutdown: watch::Sender<bool>,
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
        let (shutdown, _) = watch::channel(false);
        let thread_shutdown = shutdown.clone();
        let thread = std::thread::Builder::new()
            .name("bank-http-live-executor".to_owned())
            .spawn(move || run_local_executor(application, receiver, thread_shutdown))?;
        Ok((
            Self {
                sender,
                shutdown,
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
        if *self.shutdown.borrow() {
            return Err(BankHttpDenial::new(
                BankHttpDenialKind::Unavailable,
                BankHttpNextAction::Retry,
            ));
        }
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

    pub(super) fn shutdown_signal(&self) -> watch::Sender<bool> {
        self.shutdown.clone()
    }
}

fn run_local_executor<A>(
    application: Arc<A>,
    receiver: mpsc::Receiver<OpenAccountActivityStream>,
    shutdown: watch::Sender<bool>,
) -> io::Result<()>
where
    A: BankHttpApplicationAuthenticator,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let local = tokio::task::LocalSet::new();
    local.block_on(&runtime, dispatch_streams(application, receiver, shutdown))
}

async fn dispatch_streams<A>(
    application: Arc<A>,
    mut receiver: mpsc::Receiver<OpenAccountActivityStream>,
    shutdown: watch::Sender<bool>,
) -> io::Result<()>
where
    A: BankHttpApplicationAuthenticator,
{
    let mut shutdown_observation = shutdown.subscribe();
    let mut tasks = JoinSet::new();
    let mut first_error = None;
    loop {
        tokio::select! {
            biased;
            changed = shutdown_observation.changed() => {
                if changed.is_err() || *shutdown_observation.borrow() {
                    break;
                }
            }
            next = receiver.recv() => {
                let Some(stream) = next else { break; };
                tasks.spawn_local(run_stream(
                    Arc::clone(&application),
                    stream,
                    shutdown.subscribe(),
                ));
            }
            completed = tasks.join_next(), if !tasks.is_empty() => {
                if let Some(result) = completed {
                    record_stream_result(result, &mut first_error);
                }
            }
        }
    }
    shutdown.send_replace(true);
    receiver.close();
    while let Ok(mut pending) = receiver.try_recv() {
        send_terminal(
            &mut pending.terminal,
            BankHttpAccountActivityEvent::Closed {
                request_id: pending.request.request_id,
            },
        );
    }
    while let Some(result) = tasks.join_next().await {
        record_stream_result(result, &mut first_error);
    }
    first_error.map_or(Ok(()), Err)
}

fn record_stream_result(
    result: Result<io::Result<()>, tokio::task::JoinError>,
    first_error: &mut Option<io::Error>,
) {
    if first_error.is_none() {
        *first_error = match result {
            Ok(Ok(())) => None,
            Ok(Err(error)) => Some(error),
            Err(error) => Some(io::Error::other(error)),
        };
    }
}

async fn run_stream<A>(
    application: Arc<A>,
    stream: OpenAccountActivityStream,
    mut shutdown: watch::Receiver<bool>,
) -> io::Result<()>
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
    if *shutdown.borrow() {
        send_terminal(
            &mut terminal,
            BankHttpAccountActivityEvent::Closed { request_id },
        );
        return Ok(());
    }
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(request.deadline, cancellation.token());
    let principal = match tokio::select! {
        biased;
        _ = shutdown.changed() => {
            send_terminal(
                &mut terminal,
                BankHttpAccountActivityEvent::Closed { request_id },
            );
            return Ok(());
        }
        result = application.authenticate(request.credential, &scope) => result,
    } {
        Ok(principal) => principal,
        Err(denial) => {
            send_terminal(&mut terminal, denied(request_id, denial));
            return Ok(());
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
            return Ok(());
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
            return Ok(());
        }
    };
    if events
        .try_send(BankHttpAccountActivityEvent::Opened {
            request_id: request_id.clone(),
        })
        .is_err()
    {
        cancellation.cancel();
        return close_live_lease(lease);
    }
    let mut poll = tokio::time::interval(Duration::from_millis(20));
    let authentication_expiry =
        tokio::time::sleep_until(principal.authentication_valid_until().into());
    tokio::pin!(authentication_expiry);
    loop {
        tokio::select! {
            biased;
            _ = shutdown.changed() => {
                cancellation.cancel();
                let closed = close_live_lease(lease);
                send_terminal(
                    &mut terminal,
                    BankHttpAccountActivityEvent::Closed { request_id },
                );
                return closed;
            }
            _ = events.closed() => {
                cancellation.cancel();
                return close_live_lease(lease);
            }
            _ = &mut authentication_expiry => {
                let closed = close_live_lease(lease);
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
                return closed;
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
                        return close_live_lease(lease);
                    }
                }
            }
        }
    }
}

fn close_live_lease(lease: BankAccountActivityLiveLease<'_>) -> io::Result<()> {
    match lease.close() {
        BankApplicationLiveCloseOutcome::Completed => Ok(()),
        BankApplicationLiveCloseOutcome::Unavailable => Err(io::Error::other(
            "Bank account activity live lease did not close",
        )),
    }
}
