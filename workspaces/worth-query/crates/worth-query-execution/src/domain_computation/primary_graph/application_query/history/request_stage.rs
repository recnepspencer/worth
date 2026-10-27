//! Domain stops survive only an exact generic nested-stop handoff; direct controls win.
use super::denial::WorthQueryApplicationHistoryDenial as Denial;

pub(super) fn run<T>(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    operation: impl FnOnce(&mut worth_execution::MapKernelContext<'_, '_>) -> Result<T, Denial>,
) -> Result<T, Denial> {
    use worth_execution::{MapKernelStop, WorkCeilingDenial};
    let mut result = None;
    let outcome = execution.run_native_with_checkpoint(|context| {
        result = Some(operation(context));
    });
    match (outcome, result) {
        (Ok(_), Some(result)) => result,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
            Err(cause)
        }
        (Err(cause), _) => Err(Denial::Request(cause)),
        (Ok(_), None) => unreachable!("completed History request callback"),
    }
}

pub(super) fn checkpoint(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    units: u64,
) -> Result<(), Denial> {
    let outcome = if context.has_accounting() {
        context.checkpoint(units)
    } else {
        context.poll()
    };
    outcome.map_err(|cause| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(cause)))
}
