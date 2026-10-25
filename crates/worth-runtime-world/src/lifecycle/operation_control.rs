//! In-process scheduling at the real final product comparison boundary.
use std::num::NonZeroUsize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex, Weak,
};
use std::time::Duration;

#[path = "operation_control/currentness_hold.rs"]
mod currentness_hold;
pub(crate) use currentness_hold::ProductCurrentnessLatchForTest;
pub use currentness_hold::RuntimeWorldProductCurrentnessPauseForTest;

#[derive(Clone, Default)]
pub struct RuntimeWorldOperationControlForTest {
    pending_product_compare: Arc<ProductCompareSlot>,
    pending_currentness: Arc<ProductCurrentnessSlot>,
}
#[derive(Default)]
struct ProductCompareSlot {
    armed: AtomicBool,
    pending: Mutex<Option<ProductCompareControl>>,
}
#[derive(Default)]
struct ProductCurrentnessSlot {
    armed: AtomicBool,
    pending: Mutex<Option<Arc<ProductCurrentnessLatchForTest>>>,
}

enum ProductCompareControl {
    Pause(Arc<ExactPause>),
    Panic,
}
#[derive(Default)]
struct ExactPause {
    target: usize,
    state: Mutex<(usize, bool, Option<String>)>,
    backstop: Duration,
    changed: Condvar,
}
impl ExactPause {
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
                "BeforeProductCompareAndPublish backstop expired: missing arrival slots {:?}; arrived {}/{}; owner release={}",
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

pub struct RuntimeWorldProductComparePauseForTest {
    receiver: Weak<ProductCompareSlot>,
    latch: Arc<ExactPause>,
}
impl RuntimeWorldOperationControlForTest {
    /// Park an exact number of attempts at BeforeProductCompareAndPublish,
    /// after each last unlocked expected-head check and before acquiring its
    /// product cell.
    pub fn pause_before_product_compare_for_test(
        &self,
        attempts: NonZeroUsize,
    ) -> RuntimeWorldProductComparePauseForTest {
        let latch = Arc::new(ExactPause {
            target: attempts.get(),
            state: Mutex::new((0, false, None)),
            backstop: Duration::from_secs(120),
            changed: Condvar::new(),
        });
        let mut pending = self
            .pending_product_compare
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert!(pending.is_none(), "one product compare pause may be armed");
        *pending = Some(ProductCompareControl::Pause(Arc::clone(&latch)));
        self.pending_product_compare
            .armed
            .store(true, Ordering::Release);
        RuntimeWorldProductComparePauseForTest {
            latch,
            receiver: Arc::downgrade(&self.pending_product_compare),
        }
    }

    /// Inject one unwind after owner execution and before product movement.
    /// Armed only through this owner-issued control handle.
    pub fn panic_before_product_compare_once_for_test(&self) {
        let mut pending = self
            .pending_product_compare
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert!(
            pending.is_none(),
            "one product compare control may be armed"
        );
        *pending = Some(ProductCompareControl::Panic);
        self.pending_product_compare
            .armed
            .store(true, Ordering::Release);
    }

    pub(crate) fn before_product_compare_for_test(&self) {
        if !self.pending_product_compare.armed.load(Ordering::Acquire) {
            return;
        }
        let mut pending = self
            .pending_product_compare
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let Some(control) = pending.as_ref() else {
            return;
        };
        let latch = match control {
            ProductCompareControl::Panic => {
                pending.take();
                self.pending_product_compare
                    .armed
                    .store(false, Ordering::Release);
                panic!("injected unwind before product comparison")
            }
            ProductCompareControl::Pause(latch) => Arc::clone(latch),
        };
        let mut state = latch.state.lock().unwrap_or_else(|e| e.into_inner());
        state.0 += 1;
        if state.0 == latch.target {
            pending.take();
            self.pending_product_compare
                .armed
                .store(false, Ordering::Release);
        }
        drop(pending);
        latch.changed.notify_all();
        drop(state);
        latch.wait(false);
    }
}
impl RuntimeWorldProductComparePauseForTest {
    /// Wait for arrivals or early release; a 120-second backstop releases all waiters and panics.
    pub fn wait_for_arrivals(&self) -> bool {
        self.latch.wait(true)
    }

    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        self.latch.observe_arrivals(timeout)
    }
    pub fn release(&self) {
        self.latch.state.lock().unwrap_or_else(|e| e.into_inner()).1 = true;
        self.latch.changed.notify_all();
    }
}
impl Drop for RuntimeWorldProductComparePauseForTest {
    fn drop(&mut self) {
        // Law 2, scope: only this receiver's matching pause is cleared.
        if let Some(receiver) = self.receiver.upgrade() {
            let mut pending = receiver
                .pending
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if matches!(pending.as_ref(), Some(ProductCompareControl::Pause(latch)) if Arc::ptr_eq(latch, &self.latch))
            {
                *pending = None;
                receiver.armed.store(false, Ordering::Release);
            }
        }
        self.release();
    }
}

// Law 1, reach: issuance belongs to this sealed control file.
impl<D, I, E, Ctx, T> super::RuntimeWorldOwner<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn world_operation_control_for_test(&self) -> RuntimeWorldOperationControlForTest {
        self.root.state.operation_control.clone()
    }
}

#[cfg(test)]
mod scope_tests {
    use super::*;
    #[test]
    fn missing_arrival_backstop_releases_every_parked_attempt() {
        let control = RuntimeWorldOperationControlForTest::default();
        let mut pause =
            control.pause_before_product_compare_for_test(NonZeroUsize::new(4).unwrap());
        let short = Arc::new(ExactPause {
            target: 4,
            state: Mutex::new((0, false, None)),
            changed: Condvar::new(),
            backstop: Duration::from_millis(250),
        });
        *control.pending_product_compare.pending.lock().unwrap() =
            Some(ProductCompareControl::Pause(short.clone()));
        pause.latch = short;
        std::thread::scope(|scope| {
            let parked = (0..3)
                .map(|_| {
                    scope.spawn(|| {
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            control.before_product_compare_for_test()
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
    fn unwinding_an_unreached_compare_pause_clears_its_receiver() {
        let control = RuntimeWorldOperationControlForTest::default();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _pause = control.pause_before_product_compare_for_test(NonZeroUsize::MIN);
            panic!("exercise pause cleanup");
        }))
        .is_err());
        assert!(!control
            .pending_product_compare
            .armed
            .load(Ordering::Acquire));
        let _next = control.pause_before_product_compare_for_test(NonZeroUsize::MIN);
    }
    #[test]
    fn compare_arrivals_are_events_and_early_release_aborts_the_wait() {
        let control = RuntimeWorldOperationControlForTest::default();
        let pause = control.pause_before_product_compare_for_test(NonZeroUsize::new(2).unwrap());
        std::thread::scope(|scope| {
            for _ in 0..2 {
                scope.spawn(|| control.before_product_compare_for_test());
            }
            assert!(pause.wait_for_arrivals());
            pause.release();
        });
        let aborted = control.pause_before_product_compare_for_test(NonZeroUsize::MIN);
        aborted.release();
        assert!(!aborted.wait_for_arrivals());
    }
}
