//! Feature-only scheduling at the real final product comparison boundary.
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[derive(Clone, Default)]
pub struct RuntimeWorldOperationControl {
    pending: Arc<Mutex<Option<ProductCompareControl>>>,
}
enum ProductCompareControl {
    Pause(Arc<ProductComparePause>),
    Panic,
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
        *pending = Some(ProductCompareControl::Pause(Arc::clone(&latch)));
        RuntimeWorldProductComparePause { latch }
    }

    /// Inject one unwind after owner execution and before product movement.
    /// Available only in the explicit operation-control test build.
    pub fn panic_before_product_compare_once(&self) {
        let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        assert!(
            pending.is_none(),
            "one product compare control may be armed"
        );
        *pending = Some(ProductCompareControl::Panic);
    }

    pub(crate) fn before_product_compare(&self) {
        let control = self
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        let Some(control) = control else {
            return;
        };
        let ProductCompareControl::Pause(latch) = control else {
            panic!("injected unwind before product comparison")
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
