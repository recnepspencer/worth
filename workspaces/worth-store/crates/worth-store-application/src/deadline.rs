//! Bound existing Store deadline inputs by the genuine request's active deadline.
use worth_execution::{ExecutionRequest, WorkCeilingDenial};

#[derive(Debug)]
pub enum ApplicationHomeDeadlineDenial {
    Request(WorkCeilingDenial),
    BelowPhysicalResolution,
}

pub(crate) fn milliseconds(
    execution: &ExecutionRequest<'_, '_>,
    maximum: u64,
) -> Result<u64, ApplicationHomeDeadlineDenial> {
    execution
        .consult()
        .map_err(ApplicationHomeDeadlineDenial::Request)?;
    let Some(deadline) = execution.deadline() else {
        return Ok(maximum);
    };
    let remaining = deadline.remaining().as_millis().min(u128::from(maximum)) as u64;
    if remaining == 0 {
        execution
            .consult()
            .map_err(ApplicationHomeDeadlineDenial::Request)?;
        return Err(ApplicationHomeDeadlineDenial::BelowPhysicalResolution);
    }
    Ok(remaining)
}
