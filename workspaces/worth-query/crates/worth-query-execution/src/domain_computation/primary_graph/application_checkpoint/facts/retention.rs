//! Shared external fact custody releases its weak cache with the final holder.
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;

#[derive(Clone, Debug, Default)]
pub(in crate::domain_computation::primary_graph) struct FactRetentionStorage(
    Arc<Mutex<FactRetentionSlot>>,
);

#[derive(Debug, Default)]
pub(in crate::domain_computation::primary_graph) struct FactRetentionSlot {
    pub(in crate::domain_computation::primary_graph) allocation: Weak<FactRetentionAllocation>,
}

/// Only marks and consumed edges hold this owner strongly. Historical source
/// facts keep its weak lookup, never its invalidation-ledger reservations.
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct FactRetentionAllocation {
    pub(in crate::domain_computation::primary_graph) capacity: Arc<RetainedInvalidationCapacity>,
    origin: Weak<Mutex<FactRetentionSlot>>,
    _storage: Arc<RetainedInvalidationCapacity>,
}

impl FactRetentionStorage {
    pub(super) fn lock(&self) -> MutexGuard<'_, FactRetentionSlot> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Called with the slot locked, after both original quotes were admitted.
    /// A foreign live ledger receives an uncached, independently funded owner.
    pub(in crate::domain_computation::primary_graph) fn retain(
        &self,
        capacity: Arc<RetainedInvalidationCapacity>,
        storage: Arc<RetainedInvalidationCapacity>,
        cached: bool,
    ) -> Arc<FactRetentionAllocation> {
        Arc::new(FactRetentionAllocation {
            capacity,
            origin: if cached {
                Arc::downgrade(&self.0)
            } else {
                Weak::new()
            },
            _storage: storage,
        })
    }

    pub(super) fn bytes() -> Option<u64> {
        arc_bytes::<Mutex<FactRetentionSlot>>()?
            .checked_add(arc_bytes::<FactRetentionAllocation>()?)?
            .checked_add(arc_bytes::<RetainedInvalidationCapacity>()?)
    }
}

impl Drop for FactRetentionAllocation {
    fn drop(&mut self) {
        let Some(origin) = self.origin.upgrade() else {
            return;
        };
        let mut slot = origin
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // The final strong drop makes upgrade fail before this lock is taken.
        // A concurrent new holder may already have installed its own allocation.
        if std::ptr::eq(slot.allocation.as_ptr(), self) {
            slot.allocation = Weak::new();
        }
        // The guard is released before the capacity/storage fields are dropped.
    }
}

fn arc_bytes<T>() -> Option<u64> {
    let alignment = std::mem::align_of::<T>().max(std::mem::align_of::<usize>());
    let header = (2 * std::mem::size_of::<usize>()).div_ceil(alignment) * alignment;
    u64::try_from(
        header
            .checked_add(std::mem::size_of::<T>())?
            .div_ceil(alignment)
            .checked_mul(alignment)?,
    )
    .ok()
}
