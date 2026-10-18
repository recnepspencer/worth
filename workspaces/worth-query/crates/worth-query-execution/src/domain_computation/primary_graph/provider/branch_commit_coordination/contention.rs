//! Read-only receipt of actual contention on this runtime's existing lane.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};
use std::time::Duration;
use worth_runtime_world::facade::ProductBranchIncarnation;

#[derive(Default)]
pub(super) struct ContentionControl {
    armed: AtomicBool,
    pending: Mutex<Option<Arc<Receipt>>>,
}
#[derive(Default)]
struct Receipt {
    occurrence: Mutex<Option<ProductBranchIncarnation>>,
    changed: Condvar,
}
/// Closed certification recorder; it neither schedules nor admits a commit.
pub struct WorthQueryApplicationBranchCommitContentionForTest {
    owner: Arc<ContentionControl>,
    receipt: Arc<Receipt>,
}
impl ContentionControl {
    pub(super) fn arm(self: &Arc<Self>) -> WorthQueryApplicationBranchCommitContentionForTest {
        let receipt = Arc::new(Receipt::default());
        let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        assert!(
            pending.is_none(),
            "one receiver contention recorder may be armed"
        );
        *pending = Some(Arc::clone(&receipt));
        self.armed.store(true, Ordering::Release);
        WorthQueryApplicationBranchCommitContentionForTest {
            owner: Arc::clone(self),
            receipt,
        }
    }
    pub(super) fn armed(&self) -> bool {
        self.armed.load(Ordering::Acquire)
    }
    /// Called ONLY after this actual existing lane mutex reports WouldBlock.
    pub(super) fn observe(&self, occurrence: ProductBranchIncarnation) {
        let pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(receipt) = pending.as_ref() {
            *receipt.occurrence.lock().unwrap_or_else(|e| e.into_inner()) = Some(occurrence);
            receipt.changed.notify_all();
        }
    }
}
impl WorthQueryApplicationBranchCommitContentionForTest {
    pub fn wait_until_blocked(&self, timeout: Duration) -> bool {
        let observed = self
            .receipt
            .occurrence
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        self.receipt
            .changed
            .wait_timeout_while(observed, timeout, |value| value.is_none())
            .unwrap_or_else(|e| e.into_inner())
            .0
            .is_some()
    }
}
impl Drop for WorthQueryApplicationBranchCommitContentionForTest {
    fn drop(&mut self) {
        let mut pending = self.owner.pending.lock().unwrap_or_else(|e| e.into_inner());
        if pending
            .as_ref()
            .is_some_and(|receipt| Arc::ptr_eq(receipt, &self.receipt))
        {
            pending.take();
            self.owner.armed.store(false, Ordering::Release);
        }
    }
}
