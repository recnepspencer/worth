//! The caller's handle on one background call: poll it, wait for it, or be
//! woken when it completes. A call that never runs (or whose worker goes
//! away) still completes, with `Stopped`, so no handle can wait forever.
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

/// Why a background call produced no value.
#[derive(Debug)]
pub enum WorthQueryBackgroundDenial<Denial> {
    /// The Home closed before the call ran; nothing ran.
    HomeClosed,
    /// The call ran and refused with its ordinary typed denial.
    Denied(Denial),
    /// The worker's bounded queue was full; nothing ran. Retry later.
    Busy,
    /// The call panicked. Its Home hold was released while unwinding and the
    /// worker keeps serving later calls; any commit it reached stands.
    Panicked,
    /// The worker is not running (it could not start, or stopped) and never
    /// ran the call.
    Stopped,
    /// This handle's outcome was already taken by `try_take`.
    AlreadyTaken,
}

type Outcome<T, D> = Result<T, WorthQueryBackgroundDenial<D>>;
type Waker = Arc<dyn Fn() + Send + Sync>;

/// One background call's outcome, from an owned session's `spawn_*` methods.
#[must_use = "a pending call's outcome is lost unless taken or waited for"]
pub struct WorthQueryPending<T, D> {
    slot: Arc<Slot<T, D>>,
    taken: bool,
}

struct Slot<T, D> {
    state: Mutex<SlotState<T, D>>,
    completed: Condvar,
}

struct SlotState<T, D> {
    outcome: Option<Outcome<T, D>>,
    complete: bool,
    waker: Option<Waker>,
}

/// The worker's end of a pending call. Dropping it without completing (the
/// call was discarded unrun) completes the call with `Stopped`.
pub(crate) struct PendingCompleter<T, D> {
    slot: Option<Arc<Slot<T, D>>>,
}

/// A new pending call and the completer the worker resolves it through.
pub(crate) fn pending<T, D>() -> (WorthQueryPending<T, D>, PendingCompleter<T, D>) {
    let slot = Arc::new(Slot {
        state: Mutex::new(SlotState {
            outcome: None,
            complete: false,
            waker: None,
        }),
        completed: Condvar::new(),
    });
    (
        WorthQueryPending {
            slot: Arc::clone(&slot),
            taken: false,
        },
        PendingCompleter { slot: Some(slot) },
    )
}

impl<T, D> WorthQueryPending<T, D> {
    /// Whether the outcome is ready to take. Never blocks.
    pub fn is_ready(&self) -> bool {
        !self.taken && self.slot.lock().complete
    }

    /// Takes the outcome if the call completed; `None` while it is running
    /// (and after the outcome was taken). Never blocks.
    pub fn try_take(&mut self) -> Option<Outcome<T, D>> {
        let outcome = self.slot.lock().outcome.take();
        self.taken |= outcome.is_some();
        outcome
    }

    /// Blocks until the call completes and returns its outcome.
    pub fn wait(mut self) -> Outcome<T, D> {
        if self.taken {
            return Err(WorthQueryBackgroundDenial::AlreadyTaken);
        }
        let mut state = self.slot.lock();
        while !state.complete {
            state = self
                .slot
                .completed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        self.taken = true;
        state
            .outcome
            .take()
            .unwrap_or(Err(WorthQueryBackgroundDenial::AlreadyTaken))
    }

    /// Blocks at most `timeout`; `None` if the call is still running.
    pub fn wait_timeout(&mut self, timeout: Duration) -> Option<Outcome<T, D>> {
        if self.taken {
            return Some(Err(WorthQueryBackgroundDenial::AlreadyTaken));
        }
        let state = self.slot.lock();
        let (mut state, _) = self
            .slot
            .completed
            .wait_timeout_while(state, timeout, |state| !state.complete)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let outcome = state.outcome.take();
        self.taken |= outcome.is_some();
        outcome
    }

    /// Calls `waker` once the call completes, on the worker's thread and with
    /// no lock held (such as `egui::Context::request_repaint`). A call that
    /// already completed wakes at once, on this thread. A panicking waker is
    /// contained.
    pub fn with_waker(self, waker: impl Fn() + Send + Sync + 'static) -> Self {
        let waker: Waker = Arc::new(waker);
        let completed = {
            let mut state = self.slot.lock();
            if !state.complete {
                state.waker = Some(Arc::clone(&waker));
            }
            state.complete
        };
        if completed {
            wake(&waker);
        }
        self
    }
}

impl<T, D> WorthQueryPending<T, D> {
    /// Completes a call its worker refused before running it.
    pub(crate) fn refuse(&self, denial: WorthQueryBackgroundDenial<D>) {
        self.slot.resolve(Err(denial));
    }
}

impl<T, D> PendingCompleter<T, D> {
    /// Stores the outcome, releases waiters, then wakes outside the lock.
    pub(crate) fn complete(mut self, outcome: Outcome<T, D>) {
        if let Some(slot) = self.slot.take() {
            slot.resolve(outcome);
        }
    }
}

impl<T, D> Drop for PendingCompleter<T, D> {
    fn drop(&mut self) {
        if let Some(slot) = self.slot.take() {
            slot.resolve(Err(WorthQueryBackgroundDenial::Stopped));
        }
    }
}

impl<T, D> Slot<T, D> {
    fn lock(&self) -> MutexGuard<'_, SlotState<T, D>> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn resolve(&self, outcome: Outcome<T, D>) {
        let mut state = self.lock();
        if state.complete {
            return;
        }
        state.outcome = Some(outcome);
        state.complete = true;
        let waker = state.waker.take();
        drop(state);
        self.completed.notify_all();
        if let Some(waker) = waker {
            wake(&waker);
        }
    }
}

fn wake(waker: &Waker) {
    drop(catch_unwind(AssertUnwindSafe(|| waker())));
}
