//! Finite serialized transport workers; native Query owns every operation transition.
use super::super::protocol::{
    BankHttpDenialKind, BankHttpMutationOutcome, BankHttpMutationRequest,
};
use super::{
    authentication::BankHttpApplicationAuthenticator,
    mutation_command::{admit, reject, AdmittedMoneyCommand},
    mutation_worker,
};
use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::{mpsc, oneshot, Mutex, Semaphore};
use tokio::task::JoinHandle;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource;

pub(super) enum WorkerCommand {
    Execute {
        command: AdmittedMoneyCommand,
        queued_slot: tokio::sync::OwnedSemaphorePermit,
        cancellation: WorthQueryCancellationSource,
        response: oneshot::Sender<BankHttpMutationOutcome>,
    },
    Inspect(oneshot::Sender<usize>),
    Stop,
}
struct MutationTransport {
    application: Arc<dyn super::inbound_completion::BankRailCompletionRuntime>,
    senders: Vec<mpsc::Sender<WorkerCommand>>,
    workers: Mutex<Vec<JoinHandle<()>>>,
    queue_slots: Arc<Semaphore>,
    quiesced: Arc<AtomicBool>,
    maximum_deadline: Duration,
    stopped: AtomicBool,
}
/// Listener-independent transport custody. Each command authenticates freshly.
/// Final abandonment closes ingress; it neither settles nor disposes native obligations.
#[must_use = "retain this continuation independently of listener and shutdown error custody"]
#[derive(Clone)]
pub struct BankHttpMutationContinuation {
    transport: Arc<MutationTransport>,
}
impl BankHttpMutationContinuation {
    pub(super) fn start<A: BankHttpApplicationAuthenticator>(
        application: Arc<A>,
        queue_capacity: NonZeroUsize,
        concurrency: NonZeroUsize,
        capacity: NonZeroUsize,
        maximum_deadline: Duration,
    ) -> Self {
        let permits = Arc::new(Semaphore::new(capacity.get()));
        let quiesced = Arc::new(AtomicBool::new(false));
        let mut senders = Vec::with_capacity(concurrency.get());
        let mut workers = Vec::with_capacity(concurrency.get());
        for _ in 0..concurrency.get() {
            let (sender, receiver) = mpsc::channel(queue_capacity.get());
            workers.push(tokio::spawn(mutation_worker::run(
                Arc::clone(&application),
                receiver,
                Arc::clone(&permits),
                Arc::clone(&quiesced),
            )));
            senders.push(sender);
        }
        Self {
            transport: Arc::new(MutationTransport {
                application,
                senders,
                workers: Mutex::new(workers),
                queue_slots: Arc::new(Semaphore::new(queue_capacity.get())),
                quiesced,
                maximum_deadline,
                stopped: AtomicBool::new(false),
            }),
        }
    }
    /// Exact installed runtime remains available for independent native discovery and cleanup.
    /// This does not reconstruct transport records or promise settlement after worker failure.
    pub fn runtime(&self) -> &bank_server::BankIdentityRuntime {
        self.transport.application.runtime()
    }
    /// The same strict command contract remains usable after listener loss.
    pub async fn execute(&self, request: BankHttpMutationRequest) -> BankHttpMutationOutcome {
        let command = match admit(request, self.transport.maximum_deadline) {
            Ok(command) => command,
            Err(outcome) => return outcome,
        };
        let request_id = command.request_id.clone();
        let custody_id = command.custody_id.clone();
        let deadline = command.deadline;
        let cancellation = WorthQueryCancellationSource::new();
        let _cancel_on_drop = CancelOnDrop(cancellation.clone());
        let queued_slot = match Arc::clone(&self.transport.queue_slots).try_acquire_owned() {
            Ok(slot) => slot,
            Err(_) => return reject(Some(request_id), BankHttpDenialKind::Saturated),
        };
        let (response, receiver) = oneshot::channel();
        let slot = routing_slot(&custody_id, self.transport.senders.len());
        if let Err(error) = self.transport.senders[slot].try_send(WorkerCommand::Execute {
            command,
            queued_slot,
            cancellation,
            response,
        }) {
            match error {
                mpsc::error::TrySendError::Full(_) => {
                    return reject(Some(request_id), BankHttpDenialKind::Saturated)
                }
                mpsc::error::TrySendError::Closed(_) => {
                    return reject(Some(request_id), BankHttpDenialKind::Unavailable)
                }
            }
        }
        match tokio::time::timeout_at(deadline.into(), receiver).await {
            Ok(Ok(outcome)) => outcome,
            _ => BankHttpMutationOutcome::OutcomeUnknown {
                request_id,
                custody_id,
            },
        }
    }
    /// Joins transport workers only after every native record has been lawfully closed.
    /// Refusal returns this still-usable continuation; it does not attempt native disposal.
    pub async fn finish(self) -> Result<(), Self> {
        if !matches!(self.quiesce().await, Ok(0)) {
            return Err(self);
        }
        if self.join_closed().await.is_err() {
            return Err(self);
        }
        Ok(())
    }
    pub(super) async fn quiesce(&self) -> Result<usize, ()> {
        if self.transport.stopped.load(Ordering::Acquire) {
            return Ok(0);
        }
        self.transport.quiesced.store(true, Ordering::Release);
        let mut remaining = 0;
        for sender in &self.transport.senders {
            let (response, receiver) = oneshot::channel();
            sender
                .send(WorkerCommand::Inspect(response))
                .await
                .map_err(|_| ())?;
            remaining += receiver.await.map_err(|_| ())?;
        }
        Ok(remaining)
    }
    pub(super) async fn join_closed(&self) -> Result<(), ()> {
        if self.transport.stopped.load(Ordering::Acquire) {
            return Ok(());
        }
        for sender in &self.transport.senders {
            sender.send(WorkerCommand::Stop).await.map_err(|_| ())?;
        }
        for worker in self.transport.workers.lock().await.drain(..) {
            worker.await.map_err(|_| ())?;
        }
        self.transport.stopped.store(true, Ordering::Release);
        Ok(())
    }
}
struct CancelOnDrop(WorthQueryCancellationSource);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}
fn routing_slot(id: &str, workers: usize) -> usize {
    // Stable transport routing only; no authority or native key identity derives from this hash.
    id.bytes().fold(0usize, |hash, byte| {
        hash.wrapping_mul(31).wrapping_add(usize::from(byte))
    }) % workers
}
