//! Each actual unfinished attempt owns a funded slot for its original admitted H.
use super::{
    WorthQueryDocumentHistoryDenial as DocumentDenial, WorthQueryHistoryNavigationDenial as Denial,
};
use crate::basis::WorthQueryProductBranch;
use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex, MutexGuard},
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(in crate::domain_computation::primary_graph::product_operation::document_history) struct PendingNavigationCapacity
{
    sources: Mutex<Vec<Option<WorthQueryProductBranch>>>,
    _allocation: Option<ExecutionMemoryReservation>,
    _buffer: Option<ExecutionMemoryReservation>,
}

impl PendingNavigationCapacity {
    pub(super) fn new(
        execution: &ExecutionRequest<'_, '_>,
        maximum: NonZeroUsize,
    ) -> Result<Arc<Self>, DocumentDenial> {
        let allocation = super::super::allocation::reserve_arc::<Self>(execution)
            .map_err(DocumentDenial::Recording)?;
        let bytes = maximum
            .get()
            .checked_mul(size_of::<Option<WorthQueryProductBranch>>())
            .ok_or(DocumentDenial::RecordingIdentityExhausted)?;
        let buffer = super::super::allocation::reserve_buffer(execution, bytes)
            .map_err(DocumentDenial::Recording)?;
        let mut sources = Vec::new();
        sources
            .try_reserve_exact(maximum.get())
            .map_err(DocumentDenial::Allocation)?;
        sources.resize(maximum.get(), None);
        Ok(Arc::new(Self {
            sources: Mutex::new(sources),
            _allocation: allocation,
            _buffer: buffer,
        }))
    }

    fn sources(&self) -> MutexGuard<'_, Vec<Option<WorthQueryProductBranch>>> {
        self.sources
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    pub(super) fn check(&self) -> Result<(), Denial> {
        if self.sources().iter().any(Option::is_none) {
            Ok(())
        } else {
            Err(Denial::UnfinishedAttemptLimitExceeded)
        }
    }

    pub(super) fn acquire(
        self: &Arc<Self>,
        original: WorthQueryProductBranch,
    ) -> Result<PendingNavigationSlot, Denial> {
        let mut sources = self.sources();
        let index = sources
            .iter()
            .position(Option::is_none)
            .ok_or(Denial::UnfinishedAttemptLimitExceeded)?;
        sources[index] = Some(original);
        Ok(PendingNavigationSlot {
            capacity: Arc::clone(self),
            index,
        })
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn protects(
        &self,
        branch: WorthQueryProductBranch,
    ) -> bool {
        self.sources().contains(&Some(branch))
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn has_unfinished(
        &self,
    ) -> bool {
        self.sources().iter().any(Option::is_some)
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn bound(
        &self,
    ) -> usize {
        self.sources
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .len()
    }
}

pub(super) struct PendingNavigationSlot {
    capacity: Arc<PendingNavigationCapacity>,
    index: usize,
}

impl Drop for PendingNavigationSlot {
    fn drop(&mut self) {
        // Only bounded bookkeeping: no selection/attempt guard, owner close or
        // source reconstruction runs behind release of this original slot.
        let before = self.capacity.sources()[self.index].take();
        debug_assert!(before.is_some());
    }
}
