//! Cold support is rebuilt from performed native proposal rows, within history's budget.
use super::*;
use crate::domain_computation::primary_graph::workflow::captured_input::WorkflowCapturedInputSupport;

pub(super) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    source: EntityId,
    transition: EntityId,
    charge: &mut HistoryReconstructionCharge,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<std::sync::Arc<WorkflowCapturedInputSupport>>, WorthQueryApplicationAttemptDenial>
{
    let count = compiled.captured_input_targets(source).count();
    if count == 0 {
        return Ok(None);
    }
    charge.reserve_inventory(count)?;
    let resource =
        super::super::subject::read_subject(execution, runtime, snapshot, layout, instance, facts)?;
    crate::domain_computation::primary_graph::workflow::proposal::observe_captured_workflow_history(
        execution,
        runtime,
        snapshot,
        layout,
        compiled,
        source,
        transition,
        resource,
        facts,
        |entity, kind, locator| {
            charge.reserve_record(execution, runtime, snapshot, entity, kind, locator)
        },
    )
    .map(Some)
}
