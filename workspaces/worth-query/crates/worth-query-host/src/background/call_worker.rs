//! One owned session's call worker: a named thread draining a bounded FIFO
//! queue of calls, shared by every clone of the session.
//!
//! The queue is bounded: a full queue refuses the new call (`Busy`) before it
//! runs. The thread holds only the queue, so it stops by itself once the last
//! clone of its session is dropped and the queued calls have run.
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};

use super::pending::{WorthQueryBackgroundDenial, WorthQueryPending};
use super::thread_site::spawn_named;

/// One queued call. It completes its own pending handle, panics included.
pub(crate) type BackgroundCall = Box<dyn FnOnce() + Send>;

/// Calls waiting beyond the one running before `Busy` refuses more.
pub const DEFAULT_BACKGROUND_QUEUE_CAPACITY: usize = 64;

pub(crate) struct CallWorker {
    /// `None` when the operating system refused the thread.
    queue: Option<SyncSender<BackgroundCall>>,
}

impl CallWorker {
    pub(crate) fn start() -> Self {
        let (queue, calls) = sync_channel(DEFAULT_BACKGROUND_QUEUE_CAPACITY);
        let started = spawn_named("worth-session-calls", move || drain(&calls)).is_ok();
        Self {
            queue: started.then_some(queue),
        }
    }

    /// Queues `call`, or completes `pending` at once with why it cannot run.
    pub(crate) fn submit<T, D>(&self, pending: &WorthQueryPending<T, D>, call: BackgroundCall) {
        let Some(queue) = &self.queue else {
            pending.refuse(WorthQueryBackgroundDenial::Stopped);
            return;
        };
        // Refuse before the returned call (and its completer) drops, so the
        // handle reports why the call did not run.
        match queue.try_send(call) {
            Ok(()) => {}
            Err(TrySendError::Full(call)) => {
                pending.refuse(WorthQueryBackgroundDenial::Busy);
                drop(call);
            }
            Err(TrySendError::Disconnected(call)) => {
                pending.refuse(WorthQueryBackgroundDenial::Stopped);
                drop(call);
            }
        }
    }
}

/// Runs calls in order until every sender is gone. Each call contains its own
/// panic; containing it again here keeps the worker alive regardless.
fn drain(calls: &Receiver<BackgroundCall>) {
    while let Ok(call) = calls.recv() {
        drop(catch_unwind(AssertUnwindSafe(call)));
    }
}
