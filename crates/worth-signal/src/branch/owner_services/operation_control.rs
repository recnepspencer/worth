/// Named owner progression seams available only to deterministic test control.
///
/// The names are stable operation boundaries, not alternate operation engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalOwnerOperationBoundaryForTest {
    OwnerLifecycleAdmission,
    BranchRegistryLookup,
    BranchRegistryReservation,
    ExactBasisPreflight,
    TargetCellAdmission,
    BeforeCanonicalMovement,
    AfterCanonicalMovement,
    ForkSourceCapture,
    ForkDestinationInstallation,
    OutcomeConstruction,
    OwnerCloseBatch,
}

mod deterministic {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Condvar, Mutex, Weak};
    use std::time::{Duration, Instant};

    use super::SignalOwnerOperationBoundaryForTest;

    #[derive(Debug, Default)]
    struct SignalOwnerOperationControlState {
        active: AtomicBool,
        armed: Mutex<SignalOwnerArmedOperationControl>,
    }

    #[derive(Debug, Default)]
    struct SignalOwnerArmedOperationControl {
        pause: Option<(
            SignalOwnerOperationBoundaryForTest,
            Arc<SignalOwnerPauseLatch>,
        )>,
        panic: Option<SignalOwnerOperationBoundaryForTest>,
    }

    #[derive(Debug, Default)]
    struct SignalOwnerPauseState {
        reached: bool,
        released: bool,
    }

    #[derive(Debug, Default)]
    struct SignalOwnerPauseLatch {
        state: Mutex<SignalOwnerPauseState>,
        changed: Condvar,
    }

    #[derive(Debug, Clone)]
    pub struct SignalOwnerOperationControlForTest {
        state: Arc<SignalOwnerOperationControlState>,
    }

    #[derive(Debug)]
    pub struct SignalOwnerOperationPauseForTest {
        receiver: Weak<SignalOwnerOperationControlState>,
        latch: Arc<SignalOwnerPauseLatch>,
    }

    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "Phase 4 service operation controls consume this in-process seam"
        )
    )]
    impl SignalOwnerOperationControlForTest {
        pub(in crate::branch::owner_services) fn new() -> Self {
            Self {
                state: Arc::new(SignalOwnerOperationControlState::default()),
            }
        }

        pub fn arm_pause_once_for_test(
            &self,
            boundary: SignalOwnerOperationBoundaryForTest,
        ) -> SignalOwnerOperationPauseForTest {
            let latch = Arc::new(SignalOwnerPauseLatch::default());
            let mut armed = self
                .state
                .armed
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            assert!(armed.pause.is_none(), "only one owner pause may be armed");
            armed.pause = Some((boundary, Arc::clone(&latch)));
            self.state.active.store(true, Ordering::Release);
            SignalOwnerOperationPauseForTest {
                latch,
                receiver: Arc::downgrade(&self.state),
            }
        }

        pub fn inject_panic_once_for_test(&self, boundary: SignalOwnerOperationBoundaryForTest) {
            let mut armed = self
                .state
                .armed
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            assert!(armed.panic.is_none(), "only one owner panic may be armed");
            armed.panic = Some(boundary);
            self.state.active.store(true, Ordering::Release);
        }

        pub(in crate::branch::owner_services) fn reach(
            &self,
            boundary: SignalOwnerOperationBoundaryForTest,
        ) {
            if !self.state.active.load(Ordering::Acquire) {
                return;
            }
            let (panic_now, pause) = {
                let mut armed = self
                    .state
                    .armed
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let panic_now = armed.panic == Some(boundary);
                if panic_now {
                    armed.panic = None;
                }
                let pause = match armed.pause.as_ref() {
                    Some((armed_boundary, _)) if *armed_boundary == boundary => {
                        armed.pause.take().map(|(_, latch)| latch)
                    }
                    _ => None,
                };
                self.state.active.store(
                    armed.pause.is_some() || armed.panic.is_some(),
                    Ordering::Release,
                );
                (panic_now, pause)
            };
            if let Some(latch) = pause {
                latch.park();
            }
            if panic_now {
                panic!("injected Signal owner operation fault at {boundary:?}");
            }
        }
    }

    impl SignalOwnerPauseLatch {
        fn park(&self) {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.reached = true;
            self.changed.notify_all();
            while !state.released {
                state = self
                    .changed
                    .wait(state)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
            }
        }

        fn release(&self) {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.released = true;
            self.changed.notify_all();
        }
    }

    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "Phase 4 service operation controls consume this in-process seam"
        )
    )]
    impl SignalOwnerOperationPauseForTest {
        pub fn wait_until_reached(&self, timeout: Duration) -> bool {
            let deadline = Instant::now() + timeout;
            let mut state = self
                .latch
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            while !state.reached {
                let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                    return false;
                };
                let (next, timed) = self
                    .latch
                    .changed
                    .wait_timeout(state, remaining)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                state = next;
                if timed.timed_out() && !state.reached {
                    return false;
                }
            }
            true
        }

        pub fn release(&self) {
            self.latch.release();
        }
    }

    impl Drop for SignalOwnerOperationPauseForTest {
        fn drop(&mut self) {
            // Law 2, scope: dropping even an unreached pause disarms its receiver.
            if let Some(receiver) = self.receiver.upgrade() {
                let mut armed = receiver
                    .armed
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if armed
                    .pause
                    .as_ref()
                    .is_some_and(|(_, latch)| Arc::ptr_eq(latch, &self.latch))
                {
                    armed.pause = None;
                    receiver
                        .active
                        .store(armed.panic.is_some(), Ordering::Release);
                }
            }
            self.release();
        }
    }
    #[cfg(test)]
    mod scope_tests {
        use super::*;
        #[test]
        fn dropping_an_unreached_pause_does_not_leave_its_owner_armed() {
            let control = SignalOwnerOperationControlForTest::new();
            let pause = control.arm_pause_once_for_test(
                SignalOwnerOperationBoundaryForTest::OwnerLifecycleAdmission,
            );
            drop(pause);
            assert!(!control.state.active.load(Ordering::Acquire));
            let _next = control.arm_pause_once_for_test(
                SignalOwnerOperationBoundaryForTest::OwnerLifecycleAdmission,
            );
        }
    }
}

#[allow(
    unused_imports,
    reason = "the frozen controller contract names the pause type for service-lane consumers"
)]
pub use deterministic::{SignalOwnerOperationControlForTest, SignalOwnerOperationPauseForTest};
