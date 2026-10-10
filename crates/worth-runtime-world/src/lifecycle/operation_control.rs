//! In-process scheduling at the real final product comparison boundary.
use std::num::NonZeroUsize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex, Weak,
};
use std::time::Duration;

mod currentness_hold;
pub(crate) use currentness_hold::ProductCurrentnessLatch;
pub use currentness_hold::RuntimeWorldProductCurrentnessPause;

#[derive(Clone, Default)]
pub struct RuntimeWorldOperationControl {
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
    pending: Mutex<Option<Arc<ProductCurrentnessLatch>>>,
}

enum ProductCompareControl {
    Pause(Arc<ExactPause>),
    Panic,
}
#[derive(Default)]
struct ExactPause {
    target: usize,
    state: Mutex<(usize, bool)>,
    changed: Condvar,
}
pub struct RuntimeWorldProductComparePause {
    receiver: Weak<ProductCompareSlot>,
    latch: Arc<ExactPause>,
}
impl RuntimeWorldOperationControl {
    /// Park an exact number of attempts at BeforeProductCompareAndPublish,
    /// after each last unlocked expected-head check and before acquiring its
    /// product cell.
    pub fn pause_before_product_compare(
        &self,
        attempts: NonZeroUsize,
    ) -> RuntimeWorldProductComparePause {
        let latch = Arc::new(ExactPause {
            target: attempts.get(),
            state: Mutex::new((0, false)),
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
        RuntimeWorldProductComparePause {
            latch,
            receiver: Arc::downgrade(&self.pending_product_compare),
        }
    }

    /// Inject one unwind after owner execution and before product movement.
    /// Armed only through this owner-issued control handle.
    pub fn panic_before_product_compare_once(&self) {
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

    pub(crate) fn before_product_compare(&self) {
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
        let (state, _) = latch
            .changed
            .wait_timeout_while(state, Duration::from_secs(30), |s| !s.1)
            .unwrap_or_else(|e| e.into_inner());
        assert!(state.1, "product comparison pause exceeded release budget");
    }
}
impl RuntimeWorldProductComparePause {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        let state = self.latch.state.lock().unwrap_or_else(|e| e.into_inner());
        self.latch
            .changed
            .wait_timeout_while(state, timeout, |s| s.0 != self.latch.target)
            .unwrap_or_else(|e| e.into_inner())
            .0
             .0
            == self.latch.target
    }
    pub fn release(&self) {
        self.latch.state.lock().unwrap_or_else(|e| e.into_inner()).1 = true;
        self.latch.changed.notify_all();
    }
}
impl Drop for RuntimeWorldProductComparePause {
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
    pub fn world_operation_control_for_test(&self) -> RuntimeWorldOperationControl {
        self.root.state.operation_control.clone()
    }
}

#[cfg(test)]
mod scope_tests {
    use super::*;
    #[test]
    fn unwinding_an_unreached_compare_pause_clears_its_receiver() {
        let control = RuntimeWorldOperationControl::default();
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _pause = control.pause_before_product_compare(NonZeroUsize::MIN);
            panic!("exercise pause cleanup");
        }))
        .is_err());
        assert!(!control
            .pending_product_compare
            .armed
            .load(Ordering::Acquire));
        let _next = control.pause_before_product_compare(NonZeroUsize::MIN);
    }
}
