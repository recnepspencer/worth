use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

/// One terminal commit digest per admitted completion, shared by its clones.
/// The Signal runtime issues an admitted completion once. Releasing the last
/// capability also releases its deduplication state.
#[derive(Clone, Default)]
pub(crate) struct AsyncCompletionWritebackReceipt(Arc<Mutex<Option<Arc<str>>>>);

impl AsyncCompletionWritebackReceipt {
    pub(crate) fn try_reserve(&self) -> Option<MutexGuard<'_, Option<Arc<str>>>> {
        self.0.try_lock().ok()
    }
}

impl fmt::Debug for AsyncCompletionWritebackReceipt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<opaque>")
    }
}

impl PartialEq for AsyncCompletionWritebackReceipt {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for AsyncCompletionWritebackReceipt {}
