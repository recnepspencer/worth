//! Passive callback-attempt counts, independent of provider behavior.
use std::sync::atomic::{AtomicUsize, Ordering};
#[derive(Default)]
pub struct AccountIndexWork {
    pub(super) builds: AtomicUsize,
    pub(super) updates: AtomicUsize,
    pub(super) changes: AtomicUsize,
}
impl AccountIndexWork {
    pub(crate) fn snapshot(&self) -> (usize, usize, usize) {
        (
            self.builds.load(Ordering::Relaxed),
            self.updates.load(Ordering::Relaxed),
            self.changes.load(Ordering::Relaxed),
        )
    }
}
