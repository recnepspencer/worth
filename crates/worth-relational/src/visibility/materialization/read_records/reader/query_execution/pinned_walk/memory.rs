//! Before-growth claims on the original optional execution backing.
use super::PinnedStructuralWalkDenialKind as Denial;
use crate::execution::RelationalExecutionDenialCause as Cause;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, LeaseDenial};

pub(super) struct WalkMemory {
    scratch: Option<ExecutionMemoryReservation>,
    result: Option<ExecutionMemoryReservation>,
    scratch_bytes: u64,
    temporary_bytes: u64,
}

impl WalkMemory {
    pub(super) fn new(
        execution: &ExecutionRequest<'_, '_>,
        accounted: bool,
    ) -> Result<Self, Denial> {
        Ok(Self {
            scratch: original_reservation(execution, accounted)?,
            result: original_reservation(execution, accounted)?,
            scratch_bytes: 0,
            temporary_bytes: 0,
        })
    }

    pub(super) fn claim_scratch(&mut self, bytes: u64) -> Result<(), Denial> {
        let total = self
            .scratch_bytes
            .checked_add(bytes)
            .ok_or(Cause::ChargedBytesOverflow)?;
        resize(
            &mut self.scratch,
            total
                .checked_add(self.temporary_bytes)
                .ok_or(Cause::ChargedBytesOverflow)?,
        )?;
        self.scratch_bytes = total;
        Ok(())
    }

    pub(super) fn temporary(&mut self, bytes: u64) -> Result<(), Denial> {
        let held = self.temporary_bytes.max(bytes);
        let total = self
            .scratch_bytes
            .checked_add(held)
            .ok_or(Cause::ChargedBytesOverflow)?;
        resize(&mut self.scratch, total)?;
        self.temporary_bytes = held;
        Ok(())
    }

    pub(super) fn release_temporary(&mut self) -> Result<(), Denial> {
        resize(&mut self.scratch, self.scratch_bytes)?;
        self.temporary_bytes = 0;
        Ok(())
    }

    pub(super) fn result(&mut self, bytes: u64) -> Result<(), Denial> {
        resize(&mut self.result, bytes)
    }

    pub(super) fn take_result(&mut self) -> Option<ExecutionMemoryReservation> {
        self.result.take()
    }
}

fn original_reservation(
    execution: &ExecutionRequest<'_, '_>,
    accounted: bool,
) -> Result<Option<ExecutionMemoryReservation>, Denial> {
    let result = execution
        .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, 0))
        .map_err(crate::visibility::materialization::read_records::read_request::cause)?;
    match result {
        Ok(ticket) => Ok(Some(ticket)),
        Err(LeaseDenial::NoActiveExecutionScope) if !accounted && !execution.has_accounting() => {
            Ok(None)
        }
        Err(cause) => Err(crate::execution::denial::lease_denial(cause).into()),
    }
}

fn resize(ticket: &mut Option<ExecutionMemoryReservation>, bytes: u64) -> Result<(), Denial> {
    if let Some(ticket) = ticket {
        ticket.resize(bytes).map_err(|cause| {
            crate::execution::denial::lease_denial(LeaseDenial::MemoryExhausted(cause))
        })?;
    }
    Ok(())
}

// Rust 1.94: 11 key/value slots, 12 child pointers and parent/length/index.
pub(super) fn tree_entry<K, V>() -> Result<u64, Denial> {
    std::mem::size_of::<(K, V)>()
        .checked_mul(11)
        .and_then(|n| n.checked_add(14 * std::mem::size_of::<usize>()))
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(|| Cause::ChargedBytesOverflow.into())
}

pub(super) fn vector_bytes<T>(count: usize) -> Result<u64, Denial> {
    count
        .checked_mul(std::mem::size_of::<T>())
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(|| Cause::ChargedBytesOverflow.into())
}
