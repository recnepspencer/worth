//! One funded document-local counter; each unfinished registered attempt owns one slot.
use super::WorthQueryHistoryNavigationDenial as Denial;
use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(super) struct PendingNavigationCapacity {
    maximum: NonZeroUsize,
    unfinished: AtomicUsize,
    _allocation: Option<ExecutionMemoryReservation>,
}

impl PendingNavigationCapacity {
    pub(super) fn new(
        execution: &ExecutionRequest<'_, '_>,
        maximum: NonZeroUsize,
    ) -> Result<Arc<Self>, super::WorthQueryDocumentHistoryDenial> {
        let allocation = super::super::allocation::reserve_arc::<Self>(execution)
            .map_err(super::WorthQueryDocumentHistoryDenial::Recording)?;
        Ok(Arc::new(Self {
            maximum,
            unfinished: AtomicUsize::new(0),
            _allocation: allocation,
        }))
    }

    /// Early refusal precedes protected target/source lookup. Only acquisition
    /// in the final registration transaction reserves an actual slot.
    pub(super) fn check(&self) -> Result<(), Denial> {
        if self.unfinished.load(Ordering::Acquire) < self.maximum.get() {
            Ok(())
        } else {
            Err(Denial::UnfinishedAttemptLimitExceeded)
        }
    }

    pub(super) fn acquire(self: &Arc<Self>) -> Result<PendingNavigationSlot, Denial> {
        self.unfinished
            .try_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < self.maximum.get()).then(|| count + 1)
            })
            .map_err(|_| Denial::UnfinishedAttemptLimitExceeded)?;
        Ok(PendingNavigationSlot(Arc::clone(self)))
    }
}

pub(super) struct PendingNavigationSlot(Arc<PendingNavigationCapacity>);

impl Drop for PendingNavigationSlot {
    fn drop(&mut self) {
        // Releasing a slot never takes the document/attempt guard or performs
        // owner cleanup. Those transitions already own their actual custody.
        let before = self.0.unfinished.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(before > 0);
    }
}
