use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(test)]
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex};

use super::RelationalRuntimeAdmissionHoldDenial;

/// Admission is stopped and the owner is waiting for admitted operations.
const CLOSING: usize = 1 << (usize::BITS - 2);
/// Admission is stopped for good with nothing in flight.
const CLOSED: usize = 1 << (usize::BITS - 1);
/// Admission waits until the exclusive owner releases or seals its hold.
const HELD: usize = 1 << (usize::BITS - 3);
const STOPPED: usize = HELD | CLOSING | CLOSED;
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
    Held,
    Closing,
    Closed,
}

#[derive(Debug)]
struct RelationalRuntimeLifecycle {
    /// The stop bits and the in-flight operation count, in one word.
    ///
    /// Every transition is a single atomic step on this word, so an admission
    /// and a stop can never each miss the other: an admission's increment
    /// returns the stop bits it raced, a hold exchanges the whole word and so
    /// changes nothing when any operation is in flight, and a release's
    /// decrement returns whether a draining owner is waiting on it.
    word: AtomicUsize,
    close_wait: Mutex<()>,
    close_ready: Condvar,
    #[cfg(test)]
    test_close_start_ack: Mutex<Option<Sender<()>>>,
    #[cfg(test)]
    test_hold_wait_ack: Mutex<Option<Sender<bool>>>,
    #[cfg(test)]
    test_hold_start_pause: Mutex<Option<(Sender<()>, std::sync::mpsc::Receiver<()>)>>,
    #[cfg(test)]
    test_stopped_admission_pause: Mutex<Option<(Sender<bool>, std::sync::mpsc::Receiver<()>)>>,
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
                    #[cfg(test)]
                    test_hold_wait_ack: Mutex::new(None),
                    #[cfg(test)]
                    test_hold_start_pause: Mutex::new(None),
                    #[cfg(test)]
                    test_stopped_admission_pause: Mutex::new(None),
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
        // A forgotten hold leaves HELD behind. Closing resolves it permanently,
        // so parked and new admissions retry against CLOSED instead of waiting.
        self.lifecycle
            .word
            .fetch_and(!(CLOSING | HELD), Ordering::SeqCst);
        self.lifecycle.close_ready.notify_all();
    }

    /// Hold admission without waiting, changing nothing if work is in flight.
    pub(in crate::runtime) fn try_hold_admission(
        &self,
    ) -> Result<(), RelationalRuntimeAdmissionHoldDenial> {
        self.lifecycle
            .word
            .compare_exchange(0, HELD, Ordering::SeqCst, Ordering::SeqCst)
            .map(drop)
            .map_err(|_| RelationalRuntimeAdmissionHoldDenial::AdmissionsActive)
    }

    pub(in crate::runtime) fn release_hold(&self) {
        let _wait = self
            .lifecycle
            .close_wait
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        self.lifecycle.word.fetch_and(!HELD, Ordering::SeqCst);
        self.lifecycle.close_ready.notify_all();
    }

    pub(in crate::runtime) fn seal_held(&self) {
        let _wait = self
            .lifecycle
            .close_wait
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        // Waiting admissions may still be returning their provisional increments.
        // Preserve those counts; they never became admitted operations.
        self.lifecycle.word.fetch_or(CLOSED, Ordering::SeqCst);
        self.lifecycle.word.fetch_and(!HELD, Ordering::SeqCst);
        self.lifecycle.close_ready.notify_all();
    }

    pub(crate) fn admit(&self) -> Option<AdmittedRelationalRuntimeOperation> {
        let before = self.lifecycle.word.fetch_add(1, Ordering::SeqCst);
        debug_assert!(
            before & IN_FLIGHT < IN_FLIGHT,
            "runtime operation admission overflow"
        );
        if before & STOPPED != 0 {
            return self.admit_after_stop(before);
        }
        Some(AdmittedRelationalRuntimeOperation {
            lifecycle: Arc::clone(&self.lifecycle),
        })
    }

    #[cold]
    fn admit_after_stop(&self, mut before: usize) -> Option<AdmittedRelationalRuntimeOperation> {
        loop {
            #[cfg(test)]
            self.pause_test_stopped_admission();
            release_operation(&self.lifecycle);
            if before & HELD == 0 {
                return None;
            }
            let mut wait = self
                .lifecycle
                .close_wait
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            while self.lifecycle.word.load(Ordering::SeqCst) & HELD != 0 {
                #[cfg(test)]
                if let Some(ack) = self
                    .lifecycle
                    .test_hold_wait_ack
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .take()
                {
                    let _ = ack.send(true);
                }
                wait = self
                    .lifecycle
                    .close_ready
                    .wait(wait)
                    .unwrap_or_else(|p| p.into_inner());
            }
            drop(wait);
            before = self.lifecycle.word.fetch_add(1, Ordering::SeqCst);
            if before & STOPPED == 0 {
                return Some(AdmittedRelationalRuntimeOperation {
                    lifecycle: Arc::clone(&self.lifecycle),
                });
            }
        }
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
        } else if word & HELD != 0 {
            RelationalRuntimeAdmissionPosture::Held
        } else if word & CLOSING != 0 {
            RelationalRuntimeAdmissionPosture::Closing
        } else {
            RelationalRuntimeAdmissionPosture::Open
        }
    }

    /// Send true when parked; tests send false on completion on the same channel.
    #[cfg(test)]
    pub(crate) fn install_test_hold_wait_ack(&self, ack: Sender<bool>) {
        *self.lifecycle.test_hold_wait_ack.lock().unwrap() = Some(ack);
    }

    #[cfg(test)]
    pub(crate) fn install_test_stopped_admission_pause(
        &self,
        ack: Sender<bool>,
        resume: std::sync::mpsc::Receiver<()>,
    ) {
        *self.lifecycle.test_stopped_admission_pause.lock().unwrap() = Some((ack, resume));
    }

    #[cfg(test)]
    fn pause_test_stopped_admission(&self) {
        let hook = self
            .lifecycle
            .test_stopped_admission_pause
            .lock()
            .unwrap()
            .take();
        if let Some((ack, resume)) = hook {
            ack.send(true).unwrap();
            resume.recv().unwrap();
        }
    }

    #[cfg(test)]
    pub(crate) fn test_in_flight_count(&self) -> usize {
        self.lifecycle.word.load(Ordering::SeqCst) & IN_FLIGHT
    }

    #[cfg(test)]
    pub(crate) fn test_has_hold(&self) -> bool {
        self.lifecycle.word.load(Ordering::SeqCst) & HELD != 0
    }

    #[cfg(test)]
    pub(crate) fn install_test_hold_start_pause(
        &self,
        ack: Sender<()>,
        resume: std::sync::mpsc::Receiver<()>,
    ) {
        *self.lifecycle.test_hold_start_pause.lock().unwrap() = Some((ack, resume));
    }

    #[cfg(test)]
    pub(super) fn pause_after_test_hold_start(&self) {
        let hook = self.lifecycle.test_hold_start_pause.lock().unwrap().take();
        if let Some((ack, resume)) = hook {
            ack.send(()).unwrap();
            resume.recv().unwrap();
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
