//! An allocation observer retains only the existing weak identity.
use std::any::Any;
use std::sync::{Arc, Weak};

/// Weak identity of one owner allocation. It never extends that allocation.
#[derive(Debug, Clone)]
pub struct RelationalAllocationWitness(Weak<dyn Any + Send + Sync>);

impl RelationalAllocationWitness {
    pub(crate) fn of<T: Any + Send + Sync>(owner: &Arc<T>) -> Self {
        let weak: Weak<dyn Any + Send + Sync> =
            Arc::downgrade(owner) as Weak<dyn Any + Send + Sync>;
        Self(weak)
    }
    pub(crate) fn of_erased(owner: &Arc<dyn Any + Send + Sync>) -> Self {
        Self(Arc::downgrade(owner))
    }
    /// Whether some owner still retains the witnessed allocation.
    pub fn is_retained(&self) -> bool {
        self.0.strong_count() != 0
    }
    /// Whether both witnesses name the same allocation.
    pub fn same_allocation(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
