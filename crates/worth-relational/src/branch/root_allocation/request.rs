//! Before-growth quotes for the branch owner's concrete Arc and Box allocations.
use crate::branch::RelationalBranchRootCaptureDenial as Denial;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

// Ordinary by-value header copies do not transfer the original allocation's
// ticket. Controlled reconstruction shares Arc owners or builds admitted ones.
impl Clone for super::super::RelationalBranchRoot {
    fn clone(&self) -> Self {
        Self {
            publication_bases: Default::default(),
            id: self.id,
            regions: self.regions.clone(),
            content: self.content.clone(),
            entity_slot_count: self.entity_slot_count,
            relation_slot_count: self.relation_slot_count,
            content_accumulator: self.content_accumulator,
            output_publication_positions: self.output_publication_positions.clone(),
            schema_authority: self.schema_authority.clone(),
            execution_contracts: self.execution_contracts.clone(),
            committed: self.committed.clone(),
            managed_source_image: self.managed_source_image.clone(),
            _memory: None,
        }
    }
}

pub(in crate::branch) fn reserve_arc_memory<T>(
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<Option<ExecutionMemoryReservation>, Denial> {
    let layout = std::alloc::Layout::new::<(usize, usize)>()
        .extend(std::alloc::Layout::new::<T>())
        .map_err(|_| Denial::AllocationLayoutOverflow)?
        .0
        .pad_to_align();
    reserve(layout, request)
}

pub(in crate::branch) fn reserve_box_memory<T>(
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<Option<ExecutionMemoryReservation>, Denial> {
    reserve(std::alloc::Layout::new::<T>(), request)
}

fn reserve(
    layout: std::alloc::Layout,
    request: Option<&ExecutionRequest<'_, '_>>,
) -> Result<Option<ExecutionMemoryReservation>, Denial> {
    let Some(request) = request else {
        return Ok(None);
    };
    crate::execution::check_request(request).map_err(Denial::Request)?;
    request
        .reserve_retained_memory(
            u64::try_from(layout.size()).map_err(|_| Denial::AllocationLayoutOverflow)?,
        )
        .map_err(|cause| Denial::Request(cause.into()))
}
