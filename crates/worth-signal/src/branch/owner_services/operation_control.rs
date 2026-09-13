/// Named owner progression seams available only to deterministic test control.
///
/// The names are stable operation boundaries, not alternate operation engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalOwnerOperationBoundary {
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

#[cfg(any(test, feature = "test-operation-control"))]
mod deterministic {
    use std::sync::{Arc, Condvar, Mutex};
    use std::time::{Duration, Instant};

    use super::SignalOwnerOperationBoundary;

    #[derive(Debug, Default)]
    struct SignalOwnerOperationControlState {
        armed: Mutex<SignalOwnerArmedOperationControl>,
    }

    #[derive(Debug, Default)]
    struct SignalOwnerArmedOperationControl {
        pause: Option<(SignalOwnerOperationBoundary, Arc<SignalOwnerPauseLatch>)>,
        panic: Option<SignalOwnerOperationBoundary>,
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
    pub struct SignalOwnerOperationControl {
        state: Arc<SignalOwnerOperationControlState>,
    }

    #[derive(Debug)]
    pub struct SignalOwnerOperationPause {
        latch: Arc<SignalOwnerPauseLatch>,
    }

    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "Phase 4 service operation controls consume this feature-only seam"
        )
    )]
    impl SignalOwnerOperationControl {
        pub(in crate::branch::owner_services) fn new() -> Self {
            Self {
                state: Arc::new(SignalOwnerOperationControlState::default()),
            }
        }

        pub fn arm_pause_once(
            &self,
            boundary: SignalOwnerOperationBoundary,
        ) -> SignalOwnerOperationPause {
            let latch = Arc::new(SignalOwnerPauseLatch::default());
            let mut armed = self
                .state
                .armed
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            assert!(armed.pause.is_none(), "only one owner pause may be armed");
            armed.pause = Some((boundary, Arc::clone(&latch)));
            SignalOwnerOperationPause { latch }
        }

        pub fn inject_panic_once(&self, boundary: SignalOwnerOperationBoundary) {
            let mut armed = self
                .state
                .armed
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            assert!(armed.panic.is_none(), "only one owner panic may be armed");
            armed.panic = Some(boundary);
        }

        pub(in crate::branch::owner_services) fn reach(
            &self,
            boundary: SignalOwnerOperationBoundary,
        ) {
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
            reason = "Phase 4 service operation controls consume this feature-only seam"
        )
    )]
    impl SignalOwnerOperationPause {
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

    impl Drop for SignalOwnerOperationPause {
        fn drop(&mut self) {
            self.release();
        }
    }
}

#[cfg(any(test, feature = "test-operation-control"))]
#[allow(
    unused_imports,
    reason = "the frozen controller contract names the pause type for service-lane consumers"
)]
pub use deterministic::{SignalOwnerOperationControl, SignalOwnerOperationPause};
