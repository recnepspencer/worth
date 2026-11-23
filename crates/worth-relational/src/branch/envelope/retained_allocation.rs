//! Before-growth backing for Branch immutable Arc and Box owners.
use crate::branch::root_capture_denial::RelationalBranchRootCaptureDenial as Denial;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(crate) fn reserve_arc_memory<T>(
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
    crate::execution::request_custody::check_request(request).map_err(Denial::Request)?;
    request
        .reserve_retained_memory(
            u64::try_from(layout.size()).map_err(|_| Denial::AllocationLayoutOverflow)?,
        )
        .map_err(|cause| Denial::Request(cause.into()))
}
