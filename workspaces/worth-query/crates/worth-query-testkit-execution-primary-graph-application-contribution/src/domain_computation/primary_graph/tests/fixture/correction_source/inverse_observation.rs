//! Scoped read-only observation of the actual inverse implementation.
use std::{
    cell::RefCell,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Weak,
    },
};
thread_local! { static ACTIVE: RefCell<Option<Weak<AtomicUsize>>> = const { RefCell::new(None) }; }
pub struct LabelInverseObservation {
    count: Arc<AtomicUsize>,
}
impl LabelInverseObservation {
    pub fn begin() -> Self {
        let count = Arc::new(AtomicUsize::new(0));
        ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            assert!(active.is_none(), "one inverse recorder per thread");
            *active = Some(Arc::downgrade(&count));
        });
        Self { count }
    }
    pub fn count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}
impl Drop for LabelInverseObservation {
    fn drop(&mut self) {
        ACTIVE.with(|active| {
            active.borrow_mut().take();
        });
    }
}
pub(super) fn record() {
    ACTIVE.with(|active| {
        if let Some(count) = active.borrow().as_ref().and_then(Weak::upgrade) {
            count.fetch_add(1, Ordering::SeqCst);
        }
    });
}
