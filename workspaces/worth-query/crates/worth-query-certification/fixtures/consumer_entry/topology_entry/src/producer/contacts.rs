//! Caller-owned observations retained by the actual installed provider.
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Clone, Default)]
pub(crate) struct ProviderContacts(Arc<AtomicUsize>);

impl ProviderContacts {
    pub(crate) fn reset(&self) {
        self.0.store(0, Ordering::SeqCst);
    }

    pub(crate) fn count(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }

    pub(super) fn record(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
