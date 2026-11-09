//! Retained route allocations remain on the request that compiled their meaning.
use super::super::DerivedIndexMaintenanceDenialKind as Denial;
use super::allocation::Payment;
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, MapKernelStop, WorkCeilingDenial,
};
pub(super) fn run<T>(
    execution: &ExecutionRequest<'_, '_>,
    build: impl FnOnce(&mut Payment<'_>) -> Result<T, Denial>,
) -> Result<(T, Option<ExecutionMemoryReservation>), Denial> {
    let mut memory = execution
        .reserve_retained_memory(0)
        .map_err(|cause| Denial::Execution(cause.into()))?;
    let mut answer = None;
    let scope = execution.run_native_with_checkpoint(|context| {
        let mut pay = |units, bytes| {
            if context.has_accounting() {
                context.checkpoint(units)
            } else {
                context.poll()
            }
            .map_err(|cause| Denial::Execution(cause.into()))?;
            if let Some(memory) = &mut memory {
                let total = memory
                    .bytes()
                    .checked_add(bytes)
                    .ok_or(Denial::WorkBudgetExceeded)?;
                memory.resize(total).map_err(|cause| {
                    Denial::Execution(crate::execution::denial::lease_denial(
                        worth_execution::LeaseDenial::MemoryExhausted(cause),
                    ))
                })?;
            }
            Ok(())
        };
        answer = Some(build(&mut pay));
    });
    match (scope, answer) {
        (Ok(_), Some(answer)) => answer.map(|value| (value, memory)),
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(stop))) => {
            Err(stop)
        }
        (Err(stop), _) => Err(Denial::Execution(stop.into())),
        (Ok(_), None) => unreachable!("routing compiler completed its one native operation"),
    }
}
