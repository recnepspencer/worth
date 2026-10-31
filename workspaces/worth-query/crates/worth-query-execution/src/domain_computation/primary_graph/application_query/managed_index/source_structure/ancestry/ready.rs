//! Paid derived jump backing, published only above an already Ready ancestor chain.
use super::*;
use worth_execution::{ExecutionArray, ExecutionMemoryReservation, ExecutionRequestArrayBuilder};

pub(super) struct Ready {
    pub(super) depth: usize,
    jumps: ExecutionArray<usize>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl Ready {
    pub(super) fn derive(
        parent: Option<(usize, Arc<Self>)>,
        execution: &ExecutionRequest<'_, '_>,
        control: &Control<'_>,
        mut ancestor: impl FnMut(usize) -> Result<Arc<Self>, Stop>,
    ) -> Result<Arc<Self>, Stop> {
        let depth = match &parent {
            Some((_, parent)) => parent
                .depth
                .checked_add(1)
                .ok_or_else(|| control.refuse(Stop::CapacityOverflow))?,
            None => 0,
        };
        let levels = (usize::BITS - depth.leading_zeros()) as usize;
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .ok()
            .map(|(layout, _)| layout.pad_to_align().size())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| control.refuse(Stop::CapacityOverflow))?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| control.refuse(Stop::Request(cause)))?;
        let mut jumps = ExecutionRequestArrayBuilder::allocate(*execution, levels)
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        if let Some((parent, _)) = parent {
            jumps
                .push(parent)
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
            for bit in 1..levels {
                control.checkpoint(1)?;
                let half = jumps.elements()[bit - 1];
                let next = ancestor(half)?.jump(bit - 1, control)?;
                jumps
                    .push(next)
                    .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
            }
        }
        let jumps = jumps
            .seal()
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        Ok(Arc::new(Self {
            depth,
            jumps,
            _memory: memory,
        }))
    }
    pub(super) fn jump(&self, bit: usize, control: &Control<'_>) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        self.jumps
            .get(bit)
            .copied()
            .ok_or_else(|| control.refuse(Stop::IncompleteTraversal))
    }
}
