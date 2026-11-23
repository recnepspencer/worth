//! Compare exact runtime origins without lending the live runtime allocation.
use std::sync::{atomic::AtomicU64, Arc, Weak};

/// A weak observation of the transaction sequence assigned uniquely to one
/// Native runtime. Binding to its allocation retains neither the sequence
/// nor runtime, and it cannot issue transactions.
#[derive(Debug, Clone)]
pub(crate) struct RelationalRuntimeOrigin(Weak<AtomicU64>);

impl RelationalRuntimeOrigin {
    pub(super) fn of(sequence: &Arc<AtomicU64>) -> Self {
        Self(Arc::downgrade(sequence))
    }

    pub(crate) fn same_origin(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
