//! Collection storage and dispatch are charged through the caller's real request.
use super::*;
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, LeaseDenial, WorkCeilingDenial,
};
use WorthQueryHistoryOutputDenial as Denial;

pub(super) fn reserve(
    execution: &ExecutionRequest<'_, '_>,
    bytes: usize,
) -> Result<Option<ExecutionMemoryReservation>, Denial> {
    let bytes = u64::try_from(bytes).map_err(|_| Denial::CapacityOverflow)?;
    let memory = execution
        .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, bytes))
        .map_err(Denial::Work)?;
    match memory {
        Ok(memory) => Ok(Some(memory)),
        Err(LeaseDenial::NoActiveExecutionScope) if !execution.has_accounting() => Ok(None),
        Err(cause) => Err(Denial::Memory(cause)),
    }
}

pub(super) fn checkpoint<S: ApplicationSchema>(
    phase: &WorthQueryAdvancementPhase<'_>,
    runtime: &worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
) -> Result<(), Denial> {
    let execution = phase
        .checked_execution_request_for(runtime.product_runtime())
        .map_err(Denial::Execution)?;
    execution
        .run_native_with_checkpoint(|context| {
            if context.has_accounting() {
                context.checkpoint(4)
            } else {
                context.poll()
            }
        })
        .map_err(Denial::Work)?
        .0
        .map_err(|stop| Denial::Work(WorkCeilingDenial::Stopped(stop)))
}
