//! Construction-time reservation for actual tree nodes and their fixed pages.
use crate::execution::RelationalExecutionDenialCause as Cause;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

pub(super) fn reserve(
    request: Option<&ExecutionRequest<'_, '_>>,
    bytes: Option<u64>,
) -> Result<Option<ExecutionMemoryReservation>, Cause> {
    let bytes = bytes.ok_or(Cause::ChargedBytesOverflow)?;
    match request {
        Some(request) => {
            request.consult()?;
            request.reserve_retained_memory(bytes).map_err(Into::into)
        }
        None => Ok(None), // Existing ordinary image/maintenance construction.
    }
}

pub(super) fn array<T>(count: usize) -> Option<u64> {
    u64::try_from(std::alloc::Layout::array::<T>(count).ok()?.size()).ok()
}
