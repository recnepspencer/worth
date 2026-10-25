//! Handle-armed scheduling at real application-attempt boundaries.

use std::num::NonZeroUsize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};
use std::time::Duration;

mod caller_pass;
mod source_publication;
pub use source_publication::{
    WorthQuerySourceFactComparisonPauseForTest, WorthQuerySourcePublicationHeadForTest,
    WorthQuerySourcePublicationImageForTest, WorthQuerySourcePublicationValidationPauseForTest,
};

#[derive(Clone, Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationAttemptOperationControl
{
    pub(in crate::domain_computation::primary_graph) caller_pass:
        Arc<caller_pass::CallerPassControl>,
    pending_registration: Arc<PendingAttemptPause>,
    pending_candidate: Arc<PendingAttemptPause>,
    pending_source_validation: Arc<PendingAttemptPause>,
    pending_source_comparison: Arc<PendingAttemptPause>,
    source_validation_observations: Arc<Mutex<Vec<WorthQuerySourcePublicationImageForTest>>>,
}

#[derive(Default)]
struct PendingAttemptPause {
    armed: AtomicBool,
    pending: Mutex<Option<Arc<ExactAttemptPause>>>,
}

struct ExactAttemptPause {
    target: usize,
    state: Mutex<(usize, bool, Option<String>)>,
    backstop: Duration,
    changed: Condvar,
}

impl ExactAttemptPause {
    fn observe_arrivals(&self, timeout: Duration) -> bool {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let (state, _) = self
            .changed
            .wait_timeout_while(state, timeout, |s| s.0 != self.target && !s.1)
            .unwrap_or_else(|e| e.into_inner());
        let failure = state.2.clone();
        let reached = state.0 == self.target;
        drop(state);
        if let Some(failure) = failure {
            panic!("{failure}");
        }
        reached
    }

    fn wait(&self, arrivals: bool) -> bool {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let (mut state, expired) = self
            .changed
            .wait_timeout_while(state, self.backstop, |s| {
                !s.1 && (!arrivals || s.0 < self.target)
            })
            .unwrap_or_else(|e| e.into_inner());
        if expired.timed_out() && !state.1 && (!arrivals || state.0 < self.target) {
            state.2 = Some(format!(
                "application-attempt pause backstop expired: missing arrival slots {:?}; arrived {}/{}; owner release={}",
                (state.0 + 1..=self.target).collect::<Vec<_>>(), state.0, self.target, state.1,
            ));
            state.1 = true;
            self.changed.notify_all();
        }
        let failure = state.2.clone();
        let reached = state.0 == self.target;
        drop(state);
        if let Some(failure) = failure {
            panic!("{failure}");
        }
        reached
    }
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
        state: Mutex::new((0, false, None)),
        backstop: Duration::from_secs(120),
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
    drop(state);
    latch.wait(false);
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
    /// Wait for arrivals or early release; a 120-second backstop releases all waiters and panics.
    pub fn wait_for_arrivals(&self) -> bool {
        self.latch.wait(true)
    }

    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        self.latch.observe_arrivals(timeout)
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
        self.latch.observe_arrivals(timeout)
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

#[cfg(test)]
mod arrival_tests {
    use super::*;
    #[test]
    fn missing_arrival_backstop_releases_every_parked_attempt() {
        let control = WorthQueryApplicationAttemptOperationControl::default();
        let mut pause = control.pause_after_registration(NonZeroUsize::new(4).unwrap());
        let short = Arc::new(ExactAttemptPause {
            target: 4,
            state: Mutex::new((0, false, None)),
            changed: Condvar::new(),
            backstop: Duration::from_millis(250),
        });
        *control.pending_registration.pending.lock().unwrap() = Some(short.clone());
        pause.latch = short;
        std::thread::scope(|scope| {
            let parked = (0..3)
                .map(|_| {
                    scope.spawn(|| {
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            control.after_registration()
                        }))
                    })
                })
                .collect::<Vec<_>>();
            let observer = scope.spawn(|| {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    pause.wait_until_reached(Duration::from_secs(120))
                }))
            });
            let stopped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                pause.wait_for_arrivals()
            }));
            let message = stopped.expect_err("the fourth member never arrives");
            let message = message.downcast_ref::<String>().unwrap();
            assert!(
                message.contains("backstop expired") && message.contains("4]"),
                "{message}"
            );
            let observed = observer
                .join()
                .expect("bounded observer catches the stop")
                .expect_err("bounded observers must receive the same fatal stop");
            assert_eq!(observed.downcast_ref::<String>().unwrap(), message);
            for worker in parked {
                assert!(
                    worker.join().expect("worker catches the timeout").is_err(),
                    "all parked workers receive the stop"
                );
            }
        });
    }

    #[test]
    fn arrivals_are_observed_as_an_event_and_early_release_aborts_the_wait() {
        let control = WorthQueryApplicationAttemptOperationControl::default();
        let pause = control.pause_after_registration(NonZeroUsize::new(2).unwrap());
        std::thread::scope(|scope| {
            for _ in 0..2 {
                scope.spawn(|| control.after_registration());
            }
            assert!(pause.wait_for_arrivals());
            pause.release();
        });
        let aborted = control.pause_after_registration(NonZeroUsize::MIN);
        aborted.release();
        assert!(!aborted.wait_for_arrivals());
    }
}
