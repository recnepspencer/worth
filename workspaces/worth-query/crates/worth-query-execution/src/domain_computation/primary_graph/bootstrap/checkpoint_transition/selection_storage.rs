//! Original-request backing for selected native revalidation IDs.
use super::super::denial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial as Denial;
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, LeaseDenial, WorkCeilingDenial,
};

pub(super) fn reserve(
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ExecutionMemoryReservation, Denial> {
    execution.consult().map_err(request)?;
    execution
        .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, 0))
        .map_err(request)?
        .map_err(|cause| request(WorkCeilingDenial::Admission(cause)))
}

pub(super) fn append<T>(
    execution: &ExecutionRequest<'_, '_>,
    storage: &mut ExecutionMemoryReservation,
    selected: &mut Vec<T>,
    ids: impl ExactSizeIterator<Item = T>,
) -> Result<(), Denial> {
    execution.consult().map_err(request)?;
    let result = (|| {
        let count = selected.len().checked_add(ids.len()).ok_or_else(overflow)?;
        if count > selected.capacity() {
            let bytes = |count: usize| {
                count
                    .checked_mul(std::mem::size_of::<T>())
                    .and_then(|bytes| u64::try_from(bytes).ok())
                    .ok_or_else(overflow)
            };
            let previous = bytes(selected.capacity())?;
            // Reallocation may retain both arrays until the copy completes.
            let overlap = storage
                .bytes()
                .checked_add(bytes(count)?)
                .ok_or_else(overflow)?;
            storage.resize(overlap).map_err(memory)?;
            selected
                .try_reserve_exact(count - selected.len())
                .map_err(|_| denial("checkpoint revalidation selection allocation failed"))?;
            if selected.capacity() != count {
                return Err(denial(
                    "checkpoint revalidation selection allocation exceeded its quote",
                ));
            }
            storage.resize(overlap - previous).map_err(memory)?;
        }
        selected.extend(ids);
        Ok(())
    })();
    execution.consult().map_err(request)?;
    result
}
fn overflow() -> Denial {
    denial("checkpoint revalidation selection storage overflow")
}
fn memory(cause: worth_execution::MemoryLimitDenial) -> Denial {
    request(WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(
        cause,
    )))
}
fn request(cause: WorkCeilingDenial) -> Denial {
    super::super::super::request_read_denial::read(cause.into())
}
