//! Each actual journal insertion keeps the originating request's native memory grant.
use crate::domain_computation::primary_graph::structural_walk::memory;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

#[derive(Debug)]
pub(super) struct HistoryInsertionAllocation {
    _ticket: Option<ExecutionMemoryReservation>,
}

impl HistoryInsertionAllocation {
    pub(super) fn reserve<T>(
        execution: &ExecutionRequest<'_, '_>,
        len: usize,
        additional_bytes: usize,
    ) -> Result<Self, Cause> {
        let mut ticket = None;
        let mut bytes = u64::try_from(additional_bytes).map_err(|_| Cause::ChargedBytesOverflow)?;
        let reserved = execution.run_native_with_checkpoint(|context| {
            memory::grow_tree::<T>(
                &mut ticket,
                &mut bytes,
                len,
                execution,
                context.has_accounting(),
            )
        });
        match reserved {
            Ok((result, _)) => result?,
            Err(stop) => return Err(stop.into()),
        }
        Ok(Self { _ticket: ticket })
    }
}
