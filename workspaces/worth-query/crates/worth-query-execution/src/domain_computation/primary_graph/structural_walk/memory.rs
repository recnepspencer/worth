//! Original optional execution backing for lowering and reached-scope custody.
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, LeaseDenial, WorkCeilingDenial,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(in crate::domain_computation::primary_graph) fn reserve(
    execution: &ExecutionRequest<'_, '_>,
    accounted: bool,
    bytes: u64,
) -> Result<Option<ExecutionMemoryReservation>, Cause> {
    let reservation = execution
        .in_scope(|lease| ExecutionMemoryReservation::reserve_in_scope(lease, bytes))
        .map_err(Cause::from)?;
    match reservation {
        Ok(ticket) => Ok(Some(ticket)),
        Err(LeaseDenial::NoActiveExecutionScope) if !accounted && !execution.has_accounting() => {
            Ok(None)
        }
        Err(denial) => Err(Cause::from(WorkCeilingDenial::Admission(denial))),
    }
}

pub(in crate::domain_computation::primary_graph) fn resize(
    reservation: &mut Option<ExecutionMemoryReservation>,
    bytes: u64,
) -> Result<(), Cause> {
    if let Some(ticket) = reservation {
        ticket.resize(bytes).map_err(|cause| {
            Cause::from(WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(
                cause,
            )))
        })?;
    }
    Ok(())
}

/// Conservatively admit a BTree insertion's possible new nodes before descent.
pub(in crate::domain_computation::primary_graph) fn grow_tree<T>(
    ticket: &mut Option<ExecutionMemoryReservation>,
    bytes: &mut u64,
    len: usize,
    execution: &ExecutionRequest<'_, '_>,
    accounted: bool,
) -> Result<(), Cause> {
    let after = len.checked_add(1).ok_or(Cause::ChargedBytesOverflow)?;
    let levels = u64::from(usize::BITS - after.leading_zeros()) + 1;
    let node = 11_u64
        .checked_mul(size_of::<T>() as u64)
        .and_then(|n| n.checked_add(14 * size_of::<usize>() as u64))
        .ok_or(Cause::ChargedBytesOverflow)?;
    let next = levels
        .checked_mul(node)
        .and_then(|n| bytes.checked_add(n))
        .ok_or(Cause::ChargedBytesOverflow)?;
    if ticket.is_some() {
        resize(ticket, next)?;
    } else {
        *ticket = reserve(execution, accounted, next)?;
    }
    *bytes = next;
    Ok(())
}
