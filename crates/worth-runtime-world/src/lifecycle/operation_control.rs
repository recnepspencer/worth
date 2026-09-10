//! Feature-only scheduling at the real final product comparison boundary.
use std::num::NonZeroUsize;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[derive(Clone, Default)]
pub struct RuntimeWorldOperationControl {
    pending_product_compare: Arc<Mutex<Option<ProductCompareControl>>>,
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
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert!(pending.is_none(), "one product compare pause may be armed");
        *pending = Some(ProductCompareControl::Pause(Arc::clone(&latch)));
        RuntimeWorldProductComparePause { latch }
    }

    /// Inject one unwind after owner execution and before product movement.
    /// Available only in the explicit operation-control test build.
    pub fn panic_before_product_compare_once(&self) {
        let mut pending = self
            .pending_product_compare
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert!(
            pending.is_none(),
            "one product compare control may be armed"
        );
        *pending = Some(ProductCompareControl::Panic);
    }

    pub(crate) fn before_product_compare(&self) {
        let mut pending = self
            .pending_product_compare
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let Some(control) = pending.as_ref() else {
            return;
        };
        let latch = match control {
            ProductCompareControl::Panic => {
                pending.take();
                panic!("injected unwind before product comparison")
            }
            ProductCompareControl::Pause(latch) => Arc::clone(latch),
        };
        let mut state = latch.state.lock().unwrap_or_else(|e| e.into_inner());
        state.0 += 1;
        if state.0 == latch.target {
            pending.take();
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
        self.release();
    }
}
