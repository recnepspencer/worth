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
        next_node_lineage(&self.next).ok_or_else(|| control.refuse(Stop::CapacityOverflow))
    }
}

fn next_node_lineage(counter: &AtomicUsize) -> Option<usize> {
    let mut current = counter.load(Ordering::Relaxed);
    loop {
        let next = current.checked_add(1)?;
        match counter.compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(previous) => return Some(previous),
            Err(observed) => current = observed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_lineage_refuses_exhaustion_without_reusing_or_wrapping() {
        let counter = AtomicUsize::new(0);
        for expected in 0..256 {
            assert_eq!(next_node_lineage(&counter), Some(expected));
        }
        counter.store(usize::MAX - 1, Ordering::Relaxed);
        assert_eq!(next_node_lineage(&counter), Some(usize::MAX - 1));
        assert_eq!(counter.load(Ordering::Relaxed), usize::MAX);
        assert_eq!(next_node_lineage(&counter), None);
        assert_eq!(counter.load(Ordering::Relaxed), usize::MAX);
    }
}
