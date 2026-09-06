//! Feature-only scheduling at the real final product comparison boundary.
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[derive(Clone, Default)]
pub struct RuntimeWorldOperationControl {
    pending: Arc<Mutex<Option<Arc<ProductComparePause>>>>,
}
#[derive(Default)]
struct ProductComparePause {
    state: Mutex<(bool, bool)>,
    changed: Condvar,
}
pub struct RuntimeWorldProductComparePause {
    latch: Arc<ProductComparePause>,
}
impl RuntimeWorldOperationControl {
    /// Park one attempt at BeforeProductCompareAndPublish, after its last
    /// unlocked expected-head check and before acquiring the product cell.
    pub fn pause_before_product_compare_once(&self) -> RuntimeWorldProductComparePause {
        let latch = Arc::new(ProductComparePause::default());
        let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        assert!(pending.is_none(), "one product compare pause may be armed");
        *pending = Some(Arc::clone(&latch));
        RuntimeWorldProductComparePause { latch }
    }
    pub(crate) fn before_product_compare(&self) {
        let latch = self
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        let Some(latch) = latch else {
            return;
        };
        let mut state = latch.state.lock().unwrap_or_else(|e| e.into_inner());
        state.0 = true;
        latch.changed.notify_all();
        let (state, _) = latch
            .changed
            .wait_timeout_while(state, Duration::from_secs(30), |s| !s.1)
            .unwrap_or_else(|e| e.into_inner());
        assert!(
            state.1,
            "BeforeProductCompareAndPublish pause exceeded release budget"
        );
    }
}
impl RuntimeWorldProductComparePause {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        let state = self.latch.state.lock().unwrap_or_else(|e| e.into_inner());
        self.latch
            .changed
            .wait_timeout_while(state, timeout, |s| !s.0)
            .unwrap_or_else(|e| e.into_inner())
            .0
             .0
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
