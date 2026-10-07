//! Test-only contention at the final Product currentness read.

use super::*;
use crate::identity::ProductBranchIncarnation;

pub(crate) struct ProductCurrentnessLatch {
    occurrence: ProductBranchIncarnation,
    state: Mutex<(bool, bool, bool)>,
    changed: Condvar,
}

/// Arms one final currentness read; the real writer starts at that boundary.
pub struct RuntimeWorldProductCurrentnessPause {
    latch: Arc<ProductCurrentnessLatch>,
    pending: Arc<Mutex<Option<Arc<ProductCurrentnessLatch>>>>,
}

pub(crate) struct ProductCurrentnessHold {
    worker: Option<std::thread::JoinHandle<()>>,
    latch: Arc<ProductCurrentnessLatch>,
}

impl RuntimeWorldOperationControl {
    pub fn pause_product_currentness(
        &self,
        occurrence: ProductBranchIncarnation,
    ) -> RuntimeWorldProductCurrentnessPause {
        let latch = Arc::new(ProductCurrentnessLatch {
            occurrence,
            state: Mutex::new((false, false, false)),
            changed: Condvar::new(),
        });
        let mut pending = self
            .pending_currentness
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert!(pending.is_none(), "one currentness pause may be armed");
        *pending = Some(Arc::clone(&latch));
        RuntimeWorldProductCurrentnessPause {
            latch,
            pending: Arc::clone(&self.pending_currentness),
        }
    }

    pub(crate) fn currentness_latch(
        &self,
        occurrence: ProductBranchIncarnation,
    ) -> Option<Arc<ProductCurrentnessLatch>> {
        let mut pending = self
            .pending_currentness
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if pending
            .as_ref()
            .is_some_and(|latch| latch.occurrence == occurrence)
        {
            pending.take()
        } else {
            None
        }
    }
}

impl ProductCurrentnessLatch {
    pub(crate) fn hold_reference(
        self: &Arc<Self>,
        cell: crate::branch::ProductBranchReferenceCell,
    ) -> ProductCurrentnessHold {
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
        let hold = ProductCurrentnessHold {
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

    pub(crate) fn reader_contended(&self) {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).1 = true;
        self.changed.notify_all();
    }
}

impl RuntimeWorldProductCurrentnessPause {
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
impl Drop for RuntimeWorldProductCurrentnessPause {
    fn drop(&mut self) {
        self.release();
        let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        if pending
            .as_ref()
            .is_some_and(|latch| Arc::ptr_eq(latch, &self.latch))
        {
            pending.take();
        }
    }
}
impl Drop for ProductCurrentnessHold {
    fn drop(&mut self) {
        self.latch.state.lock().unwrap_or_else(|e| e.into_inner()).2 = true;
        self.latch.changed.notify_all();
        if let Some(worker) = self.worker.take() {
            if let Err(payload) = worker.join() {
                if !std::thread::panicking() {
                    std::panic::resume_unwind(payload);
                }
            }
        }
    }
}
