use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(test)]
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex};

use super::RelationalRuntimeSealDenial;

/// Admission is stopped and the owner is waiting for admitted operations.
const CLOSING: usize = 1 << (usize::BITS - 2);
/// Admission is stopped for good with nothing in flight.
const CLOSED: usize = 1 << (usize::BITS - 1);
const STOPPED: usize = CLOSING | CLOSED;
const IN_FLIGHT: usize = !STOPPED;

/// Drop-governed lifecycle authority shared by every independently borrowable
/// service issued by one Relational runtime.
#[derive(Debug)]
pub(in crate::runtime) struct RelationalRuntimeOwner {
    binding: RelationalRuntimeOwnerBinding,
}

/// Cloneable lifecycle binding carried by narrow runtime-owned services.
#[derive(Debug, Clone)]
pub(crate) struct RelationalRuntimeOwnerBinding {
    lifecycle: Arc<RelationalRuntimeLifecycle>,
}

/// Where one runtime owner's admission stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationalRuntimeAdmissionPosture {
    Open,
    Closing,
    Closed,
}

#[derive(Debug)]
struct RelationalRuntimeLifecycle {
    /// The stop bits and the in-flight operation count, in one word.
    ///
    /// Every transition is a single atomic step on this word, so an admission
    /// and a stop can never each miss the other: an admission's increment
    /// returns the stop bits it raced, a seal exchanges the whole word and so
    /// changes nothing when any operation is in flight, and a release's
    /// decrement returns whether a draining owner is waiting on it.
    word: AtomicUsize,
    close_wait: Mutex<()>,
    close_ready: Condvar,
    #[cfg(test)]
    test_close_start_ack: Mutex<Option<Sender<()>>>,
}

#[derive(Debug)]
pub(crate) struct AdmittedRelationalRuntimeOperation {
    lifecycle: Arc<RelationalRuntimeLifecycle>,
}

impl RelationalRuntimeOwner {
    pub(in crate::runtime) fn new() -> Self {
        Self {
            binding: RelationalRuntimeOwnerBinding {
                lifecycle: Arc::new(RelationalRuntimeLifecycle {
                    word: AtomicUsize::new(0),
                    close_wait: Mutex::new(()),
                    close_ready: Condvar::new(),
                    #[cfg(test)]
                    test_close_start_ack: Mutex::new(None),
                }),
            },
        }
    }

    pub(super) fn binding(&self) -> RelationalRuntimeOwnerBinding {
        self.binding.clone()
    }
}

impl RelationalRuntimeOwnerBinding {
    /// Stop admitting operations and wait for the admitted ones to return.
    ///
    /// This is owner authority, so it stays reachable only inside the runtime
    /// module tree: a narrow service carries this binding to admit work and can
    /// never use it to close the runtime it borrows.
    pub(in crate::runtime) fn close(&self) {
        self.lifecycle.word.fetch_or(CLOSING, Ordering::SeqCst);
        #[cfg(test)]
        self.acknowledge_test_close_start();
        let mut wait = self
            .lifecycle
            .close_wait
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while self.lifecycle.word.load(Ordering::SeqCst) & IN_FLIGHT != 0 {
            wait = self
                .lifecycle
                .close_ready
                .wait(wait)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        // The drain is over and this owner was its only waiter: close is
        // reached once, through the owner's exclusive handle. Setting the
        // closed bit before clearing the drain bit keeps admission stopped
        // throughout, and with the drain bit gone an admission denied by a
        // closed owner returns without taking the drain lock.
        self.lifecycle.word.fetch_or(CLOSED, Ordering::SeqCst);
        self.lifecycle.word.fetch_and(!CLOSING, Ordering::SeqCst);
    }

    /// Stop admission for good, in place, only when nothing is in flight.
    ///
    /// Unlike [`Self::close`] this never waits. The whole word is exchanged in
    /// one step, so a refused attempt changes nothing: admission was never
    /// stopped, and no concurrent admission is denied because of it.
    ///
    /// Seal and close are both owner authority reached through the owner's
    /// exclusive handle, so the exchange can only fail on operations in flight.
    pub(in crate::runtime) fn try_seal(&self) -> Result<(), RelationalRuntimeSealDenial> {
        self.lifecycle
            .word
            .compare_exchange(0, CLOSED, Ordering::SeqCst, Ordering::SeqCst)
            .map(drop)
            .map_err(|_| RelationalRuntimeSealDenial::AdmissionsActive)
    }

    pub(crate) fn admit(&self) -> Option<AdmittedRelationalRuntimeOperation> {
        let before = self.lifecycle.word.fetch_add(1, Ordering::SeqCst);
        debug_assert!(
            before & IN_FLIGHT < IN_FLIGHT,
            "runtime operation admission overflow"
        );
        if before & STOPPED != 0 {
            release_operation(&self.lifecycle);
            return None;
        }
        Some(AdmittedRelationalRuntimeOperation {
            lifecycle: Arc::clone(&self.lifecycle),
        })
    }

    /// Observe where this owner's admission stands without admitting any work.
    ///
    /// This is descriptive state only. It carries no close authority and does
    /// not increment or otherwise participate in the in-flight drain. A sealed
    /// owner answers closed while its state is still alive.
    pub(crate) fn admission_posture(&self) -> RelationalRuntimeAdmissionPosture {
        let word = self.lifecycle.word.load(Ordering::SeqCst);
        if word & CLOSED != 0 {
            RelationalRuntimeAdmissionPosture::Closed
        } else if word & CLOSING != 0 {
            RelationalRuntimeAdmissionPosture::Closing
        } else {
            RelationalRuntimeAdmissionPosture::Open
        }
    }

    #[cfg(test)]
    pub(crate) fn install_test_close_start_ack(&self, ack: Sender<()>) {
        let mut hook = self
            .lifecycle
            .test_close_start_ack
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *hook = Some(ack);
    }

    #[cfg(test)]
    fn acknowledge_test_close_start(&self) {
        let hook = self
            .lifecycle
            .test_close_start_ack
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
        if let Some(hook) = hook {
            let _ = hook.send(());
        }
    }
}

impl Drop for AdmittedRelationalRuntimeOperation {
    fn drop(&mut self) {
        release_operation(&self.lifecycle);
    }
}

/// Return one admission, and wake a draining owner when it was the last.
///
/// A refused admission returns its own increment through here too, so a drain
/// that saw that increment is always woken once it is gone.
fn release_operation(lifecycle: &RelationalRuntimeLifecycle) {
    let before = lifecycle.word.fetch_sub(1, Ordering::SeqCst);
    debug_assert!(
        before & IN_FLIGHT > 0,
        "runtime operation admission underflow"
    );
    if before & IN_FLIGHT == 1 && before & CLOSING != 0 {
        let _wait = lifecycle
            .close_wait
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        lifecycle.close_ready.notify_all();
    }
}
