//! Handle-armed scheduling at real application-attempt boundaries.

use std::num::NonZeroUsize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};
use std::time::Duration;

mod caller_pass;

#[derive(Clone, Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationAttemptOperationControl
{
    pub(in crate::domain_computation::primary_graph) caller_pass:
        Arc<caller_pass::CallerPassControl>,
    pending_registration: Arc<PendingAttemptPause>,
    pending_candidate: Arc<PendingAttemptPause>,
}

#[derive(Default)]
struct PendingAttemptPause {
    armed: AtomicBool,
    pending: Mutex<Option<Arc<ExactAttemptPause>>>,
}

struct ExactAttemptPause {
    target: usize,
    state: Mutex<(usize, bool)>,
    changed: Condvar,
}

pub struct WorthQueryApplicationAttemptRegistrationPauseForTest {
    control: WorthQueryApplicationAttemptOperationControl,
    latch: Arc<ExactAttemptPause>,
}

pub struct WorthQueryApplicationCandidatePreparationPauseForTest {
    control: WorthQueryApplicationAttemptOperationControl,
    latch: Arc<ExactAttemptPause>,
}

impl WorthQueryApplicationAttemptOperationControl {
    pub(super) fn pause_after_registration(
        &self,
        attempts: NonZeroUsize,
    ) -> WorthQueryApplicationAttemptRegistrationPauseForTest {
        let latch = arm(&self.pending_registration, attempts);
        WorthQueryApplicationAttemptRegistrationPauseForTest {
            control: self.clone(),
            latch,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn after_registration(&self) {
        reach(&self.pending_registration);
    }

    pub(super) fn pause_after_candidate_preparation(
        &self,
        attempts: NonZeroUsize,
    ) -> WorthQueryApplicationCandidatePreparationPauseForTest {
        let latch = arm(&self.pending_candidate, attempts);
        WorthQueryApplicationCandidatePreparationPauseForTest {
            control: self.clone(),
            latch,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn after_candidate_preparation(&self) {
        reach(&self.pending_candidate);
    }

    fn clear_registration(&self, latch: &Arc<ExactAttemptPause>) {
        clear(&self.pending_registration, latch);
    }

    fn clear_candidate(&self, latch: &Arc<ExactAttemptPause>) {
        clear(&self.pending_candidate, latch);
    }
}

fn arm(slot: &PendingAttemptPause, attempts: NonZeroUsize) -> Arc<ExactAttemptPause> {
    let latch = Arc::new(ExactAttemptPause {
        target: attempts.get(),
        state: Mutex::new((0, false)),
        changed: Condvar::new(),
    });
    let mut pending = slot.pending.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        pending.is_none(),
        "one attempt pause may be armed per boundary"
    );
    *pending = Some(Arc::clone(&latch));
    slot.armed.store(true, Ordering::Release);
    latch
}

fn reach(slot: &PendingAttemptPause) {
    if !slot.armed.load(Ordering::Acquire) {
        return;
    }
    let mut pending = slot.pending.lock().unwrap_or_else(|e| e.into_inner());
    let Some(latch) = pending.as_ref().map(Arc::clone) else {
        return;
    };
    let mut state = latch.state.lock().unwrap_or_else(|e| e.into_inner());
    state.0 += 1;
    if state.0 == latch.target {
        pending.take();
        slot.armed.store(false, Ordering::Release);
    }
    drop(pending);
    latch.changed.notify_all();
    let (state, _) = latch
        .changed
        .wait_timeout_while(state, Duration::from_secs(30), |state| !state.1)
        .unwrap_or_else(|e| e.into_inner());
    assert!(state.1, "application attempt pause exceeded release budget");
}

fn clear(slot: &PendingAttemptPause, latch: &Arc<ExactAttemptPause>) {
    let mut pending = slot.pending.lock().unwrap_or_else(|e| e.into_inner());
    if pending
        .as_ref()
        .is_some_and(|armed| Arc::ptr_eq(armed, latch))
    {
        pending.take();
        slot.armed.store(false, Ordering::Release);
    }
}

impl WorthQueryApplicationAttemptRegistrationPauseForTest {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        let state = self.latch.state.lock().unwrap_or_else(|e| e.into_inner());
        self.latch
            .changed
            .wait_timeout_while(state, timeout, |state| state.0 != self.latch.target)
            .unwrap_or_else(|e| e.into_inner())
            .0
             .0
            == self.latch.target
    }

    pub fn release(&self) {
        self.control.clear_registration(&self.latch);
        self.latch.state.lock().unwrap_or_else(|e| e.into_inner()).1 = true;
        self.latch.changed.notify_all();
    }
}

impl Drop for WorthQueryApplicationAttemptRegistrationPauseForTest {
    fn drop(&mut self) {
        self.release();
    }
}

impl WorthQueryApplicationCandidatePreparationPauseForTest {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        let state = self.latch.state.lock().unwrap_or_else(|e| e.into_inner());
        self.latch
            .changed
            .wait_timeout_while(state, timeout, |state| state.0 != self.latch.target)
            .unwrap_or_else(|e| e.into_inner())
            .0
             .0
            == self.latch.target
    }

    pub fn release(&self) {
        self.control.clear_candidate(&self.latch);
        self.latch.state.lock().unwrap_or_else(|e| e.into_inner()).1 = true;
        self.latch.changed.notify_all();
    }
}

impl Drop for WorthQueryApplicationCandidatePreparationPauseForTest {
    fn drop(&mut self) {
        self.release();
    }
}
