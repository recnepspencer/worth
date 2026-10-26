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
