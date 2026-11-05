//! Moved whole from `domain_computation::primary_graph::content::retention_work`: the items below and every impl they own.
use worth_relational::facade::content::RelationalContentDenial as Denial;

pub(in crate::domain_computation) fn checkpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let mut result = None;
    let handoff = execution.run_native_with_checkpoint(|context| {
        result = Some(if context.has_accounting() {
            context.checkpoint(1)
        } else {
            context.poll()
        });
    });
    match (handoff, result) {
        (_, Some(Err(cause))) => Err(Denial::Execution(cause.into())),
        (Err(cause), _) => Err(Denial::Execution(cause.into())),
        (Ok(_), Some(Ok(()))) => Ok(()),
        (Ok(_), None) => unreachable!("completed retention checkpoint invoked its callback"),
    }
}
