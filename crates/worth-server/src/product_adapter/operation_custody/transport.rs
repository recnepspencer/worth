//! Bounded ingress crosses threads; native operation custody never does.
use super::contract::WorthServerQuerySourceAdapter as SourceWire;
use super::{
    protocol::{self, ControlRefusal, WorthServerSourceReply as Reply},
    WorthServerSourceCustodyLimits,
};
use crate::{
    WorthServerProductAdapterExecutionError as Error, WorthServerProductApplicationAdapter,
    WorthServerProductOperationDenial, WorthServerProductOperationSuccess,
    WorthServerScheduledProductOperation,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

pub(super) enum Job {
    Execute {
        operation: Box<WorthServerScheduledProductOperation>,
        reply: SyncSender<Result<Reply, Error>>,
    },
    Finish {
        reply: SyncSender<bool>,
    },
}
struct Transport {
    sender: SyncSender<Job>,
    quiesced: Arc<AtomicBool>,
    thread: Mutex<WorkerLifecycle>,
}
enum WorkerLifecycle {
    Running(JoinHandle<super::worker::WorkerExit>),
    ClosedEmpty,
    Failed,
}
/// Listener-independent process-local custody. Final abandonment performs no cleanup I/O.
#[derive(Clone)]
#[must_use = "retain native source-command custody independently of its listener"]
pub struct WorthServerSourceContinuation {
    transport: Arc<Transport>,
    execute: fn(
        &Transport,
        &WorthServerScheduledProductOperation,
    ) -> Result<WorthServerProductOperationSuccess, Error>,
}

pub(super) fn start<C: SourceWire>(
    application: Arc<C>,
    limits: WorthServerSourceCustodyLimits,
) -> std::io::Result<WorthServerSourceContinuation> {
    let (sender, receiver) = mpsc::sync_channel(limits.queued_commands.get());
    let quiesced = Arc::new(AtomicBool::new(false));
    let worker_quiesced = Arc::clone(&quiesced);
    let thread = std::thread::Builder::new()
        .name(format!("query-{}-custody", C::NAMESPACE))
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            super::worker::run::<C>(
                application.as_ref(),
                &receiver,
                limits.live_operations.get(),
                worker_quiesced.as_ref(),
            )
        })?;
    Ok(WorthServerSourceContinuation {
        transport: Arc::new(Transport {
            sender,
            quiesced,
            thread: Mutex::new(WorkerLifecycle::Running(thread)),
        }),
        execute: execute::<C>,
    })
}
impl WorthServerSourceContinuation {
    /// Quiesces new creates and joins only when native custody is empty.
    /// # Errors
    /// Returns this same continuation on outstanding custody, saturation or uncertainty.
    pub fn try_finish(self) -> Result<(), Self> {
        self.transport.quiesced.store(true, Ordering::Release);
        let (reply, answer) = mpsc::sync_channel(1);
        let acknowledged = self
            .transport
            .sender
            .try_send(Job::Finish { reply })
            .is_ok()
            && answer.recv_timeout(Duration::from_secs(5)) == Ok(true);
        let Ok(mut handle) = self.transport.thread.lock() else {
            return Err(self.clone());
        };
        match &*handle {
            WorkerLifecycle::ClosedEmpty => return Ok(()),
            WorkerLifecycle::Failed => return Err(self.clone()),
            WorkerLifecycle::Running(thread) if !acknowledged && !thread.is_finished() => {
                return Err(self.clone());
            }
            WorkerLifecycle::Running(_) => {}
        }
        let WorkerLifecycle::Running(thread) =
            std::mem::replace(&mut *handle, WorkerLifecycle::Failed)
        else {
            unreachable!("only an authentic worker is joined")
        };
        match thread.join() {
            Ok(super::worker::WorkerExit::ClosedEmpty) => {
                *handle = WorkerLifecycle::ClosedEmpty;
                Ok(())
            }
            Ok(super::worker::WorkerExit::Abandoned) | Err(_) => Err(self.clone()),
        }
    }
}
impl WorthServerProductApplicationAdapter for WorthServerSourceContinuation {
    fn execute(
        &self,
        operation: &WorthServerScheduledProductOperation,
    ) -> Result<WorthServerProductOperationSuccess, Error> {
        (self.execute)(&self.transport, operation)
    }
}
fn execute<C: SourceWire>(
    transport: &Transport,
    operation: &WorthServerScheduledProductOperation,
) -> Result<WorthServerProductOperationSuccess, Error> {
    let command = protocol::decode::<C>(operation.plan().payload()).map_err(Error::denied)?;
    let custody_id = command.custody_id().to_owned();
    let caller = operation
        .plan()
        .operation_admission()
        .operation_request()
        .resolved_request_context()
        .request_context()
        .authenticated_principal()
        .admitted_transport_caller()
        .ok_or_else(|| {
            Error::denied(WorthServerProductOperationDenial::new(
                "source.transport_identity",
                "an admitted caller is required",
            ))
        })?;
    let deadline = caller
        .query_request_scope()
        .and_then(|bound| bound.request_scope().deadline())
        .ok_or_else(|| {
            Error::denied(WorthServerProductOperationDenial::new(
                "source.request_deadline",
                "a live verifier-bound deadline is required",
            ))
        })?;
    let (reply, answer) = mpsc::sync_channel(1);
    let result = match transport.sender.try_send(Job::Execute {
        operation: Box::new(operation.clone()),
        reply,
    }) {
        Err(mpsc::TrySendError::Full(_)) => Reply::NotExecuted {
            custody_id,
            reason: ControlRefusal::QueueFull,
        },
        Err(mpsc::TrySendError::Disconnected(_)) => Reply::NotExecuted {
            custody_id,
            reason: ControlRefusal::TransportUnavailable,
        },
        Ok(()) => answer
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .unwrap_or(Ok(Reply::OutcomeUnknown { custody_id }))?,
    };
    WorthServerProductOperationSuccess::publish_json(
        super::contract::operation::<C>(),
        &protocol::result_contract(C::RESULT),
        &protocol::PublishedReply {
            value: &result,
            identity: C::RESULT,
        },
    )
    .map_err(Error::invalid_result_artifact)
}
