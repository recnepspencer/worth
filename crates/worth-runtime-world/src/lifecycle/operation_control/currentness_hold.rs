//! In-process contention at the final Product currentness read.

use super::*;
use crate::identity::ProductBranchIncarnation;

pub(crate) struct ProductCurrentnessLatchForTest {
    occurrence: ProductBranchIncarnation,
    state: Mutex<(bool, bool, bool)>,
    changed: Condvar,
}

/// Arms one final currentness read; the real writer starts at that boundary.
pub struct RuntimeWorldProductCurrentnessPauseForTest {
    latch: Arc<ProductCurrentnessLatchForTest>,
    pending: Arc<ProductCurrentnessSlot>,
}

pub(crate) struct ProductCurrentnessHoldForTest {
    worker: Option<std::thread::JoinHandle<()>>,
    latch: Arc<ProductCurrentnessLatchForTest>,
}

impl RuntimeWorldOperationControlForTest {
    pub fn pause_product_currentness_for_test(
        &self,
        occurrence: ProductBranchIncarnation,
    ) -> RuntimeWorldProductCurrentnessPauseForTest {
        let latch = Arc::new(ProductCurrentnessLatchForTest {
            occurrence,
            state: Mutex::new((false, false, false)),
            changed: Condvar::new(),
        });
        let mut pending = self
            .pending_currentness
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert!(pending.is_none(), "one currentness pause may be armed");
        *pending = Some(Arc::clone(&latch));
        self.pending_currentness
            .armed
            .store(true, Ordering::Release);
        RuntimeWorldProductCurrentnessPauseForTest {
            latch,
            pending: Arc::clone(&self.pending_currentness),
        }
    }

    pub(crate) fn currentness_latch_for_test(
        &self,
        occurrence: ProductBranchIncarnation,
    ) -> Option<Arc<ProductCurrentnessLatchForTest>> {
        if !self.pending_currentness.armed.load(Ordering::Acquire) {
            return None;
        }
        let mut pending = self
            .pending_currentness
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if pending
            .as_ref()
            .is_some_and(|latch| latch.occurrence == occurrence)
        {
            self.pending_currentness
                .armed
                .store(false, Ordering::Release);
            pending.take()
        } else {
            None
        }
    }
}

impl ProductCurrentnessLatchForTest {
    pub(crate) fn hold_reference_for_test(
        self: &Arc<Self>,
        cell: crate::branch::ProductBranchReferenceCell,
    ) -> ProductCurrentnessHoldForTest {
        let latch = Arc::clone(self);
        let worker = std::thread::spawn(move || {
            let guard = cell.hold_for_test();
            let mut state = latch.state.lock().unwrap_or_else(|e| e.into_inner());
            state.0 = true;
            latch.changed.notify_all();
            let (state, _) = latch
                .changed
                .wait_timeout_while(state, Duration::from_secs(30), |state| !state.2)
                .unwrap_or_else(|e| e.into_inner());
            let released = state.2;
            drop(state);
            drop(guard);
            assert!(released, "currentness hold exceeded release budget");
        });
        let hold = ProductCurrentnessHoldForTest {
            worker: Some(worker),
            latch: Arc::clone(self),
        };
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let (state, _) = self
            .changed
            .wait_timeout_while(state, Duration::from_secs(5), |state| !state.0)
            .unwrap_or_else(|e| e.into_inner());
        assert!(state.0, "currentness writer did not acquire the reference");
        hold
    }

    pub(crate) fn reader_contended_for_test(&self) {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).1 = true;
        self.changed.notify_all();
    }
}

impl RuntimeWorldProductCurrentnessPauseForTest {
    pub fn wait_until_reader_contended(&self, timeout: Duration) -> bool {
        let state = self.latch.state.lock().unwrap_or_else(|e| e.into_inner());
        self.latch
            .changed
            .wait_timeout_while(state, timeout, |state| !state.1)
            .unwrap_or_else(|e| e.into_inner())
            .0
             .1
    }
    pub fn release(&self) {
        self.latch.state.lock().unwrap_or_else(|e| e.into_inner()).2 = true;
        self.latch.changed.notify_all();
    }
}
impl Drop for RuntimeWorldProductCurrentnessPauseForTest {
    fn drop(&mut self) {
        self.release();
        let mut pending = self
            .pending
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if pending
            .as_ref()
            .is_some_and(|latch| Arc::ptr_eq(latch, &self.latch))
        {
            pending.take();
            self.pending.armed.store(false, Ordering::Release);
        }
    }
}
impl Drop for ProductCurrentnessHoldForTest {
    fn drop(&mut self) {
        self.latch.state.lock().unwrap_or_else(|e| e.into_inner()).2 = true;
        self.latch.changed.notify_all();
        if let Some(worker) = self.worker.take()
            && let Err(payload) = worker.join()
            && !std::thread::panicking()
        {
            std::panic::resume_unwind(payload);
        }
    }
}
