//! Keep a native owner's typed refusal through only a generic nested handoff.
use super::RelationalRecordHistoryDenial as Denial;
use worth_execution::{ExecutionRequest, MapKernelContext, MapKernelStop, WorkCeilingDenial};

pub(super) fn run<T>(
    execution: ExecutionRequest<'_, '_>,
    operation: impl FnOnce(&mut MapKernelContext<'_, '_>) -> Result<T, Denial>,
) -> Result<T, Denial> {
    let mut value = None;
    let outcome = execution.run_native_with_checkpoint(|context| {
        value = Some(operation(context));
    });
    match (outcome, value) {
        (Ok(_), Some(value)) => value,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
            Err(cause)
        }
        (Err(cause), _) => Err(Denial::Request(cause)),
        (Ok(_), None) => unreachable!("completed native history callback"),
    }
}
