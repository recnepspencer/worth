//! Descriptive node coordinates are never reused across isolated sibling edits.
use super::{Stop, WorthQueryManagedIndexControl};
use std::{
    alloc::Layout,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(super) struct NodeLineage {
    next: AtomicUsize,
    _memory: Option<ExecutionMemoryReservation>,
}

impl NodeLineage {
    pub(super) fn retain(
        execution: &ExecutionRequest<'_, '_>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Arc<Self>, Stop> {
        control.checkpoint(1)?;
        let bytes = Layout::new::<Self>()
            .extend(Layout::new::<[usize; 2]>())
            .map_err(|_| control.refuse(Stop::CapacityOverflow))?
            .0
            .pad_to_align()
            .size();
        let bytes = u64::try_from(bytes).map_err(|_| control.refuse(Stop::CapacityOverflow))?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| control.refuse(Stop::Request(cause)))?;
        Ok(Arc::new(Self {
            next: AtomicUsize::new(0),
            _memory: memory,
        }))
    }

    pub(super) fn issue(&self, control: &WorthQueryManagedIndexControl<'_>) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        self.next
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .map_err(|_| control.refuse(Stop::CapacityOverflow))
    }
}
