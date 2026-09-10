use worth_query_execution::facade::runtime::{
    WorthQueryDirectGraphStepOutcome, WorthQueryManagedGraphCallRequest,
    WorthQueryRunningDirectRun, WorthQueryRunningWorkflowRun, WorthQueryWorkflowGraphStepOutcome,
};

use super::WorthQueryBoundGraphExecutionReceipt;
use crate::domain_installation::WorthQueryGraphProviderCallKind;

pub(super) fn execute_direct_graph(
    running: WorthQueryRunningDirectRun,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    kind: WorthQueryGraphProviderCallKind,
    scope_identity: &str,
) -> Result<
    (
        WorthQueryRunningDirectRun,
        WorthQueryBoundGraphExecutionReceipt,
    ),
    (
        String,
        worth_query_execution::facade::runtime::WorthQueryDirectRunTerminal,
    ),
> {
    let active = running
        .begin_graph_execution(
            graph,
            WorthQueryManagedGraphCallRequest::new(kind, scope_identity),
        )
        .map_err(|failure| {
            let detail = failure.detail().to_owned();
            (detail, failure.into_running().abandon())
        })?;
    let mut outcome = active.advance();
    loop {
        outcome = match outcome {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused.advance(),
            WorthQueryDirectGraphStepOutcome::ChunkReady(chunk) => chunk.acknowledge(),
            WorthQueryDirectGraphStepOutcome::Completed(completed) => {
                let receipt = completed.receipt().clone();
                return Ok((completed.into_running(), receipt));
            }
            WorthQueryDirectGraphStepOutcome::Cancelled(terminal)
            | WorthQueryDirectGraphStepOutcome::TimedOut(terminal)
            | WorthQueryDirectGraphStepOutcome::Exhausted(terminal)
            | WorthQueryDirectGraphStepOutcome::Degraded(terminal)
            | WorthQueryDirectGraphStepOutcome::Failed(terminal) => {
                let detail = format!(
                    "managed graph execution terminated as {:?}",
                    terminal.kind()
                );
                return Err((detail, terminal));
            }
        };
    }
}

pub(super) fn execute_workflow_graph(
    running: WorthQueryRunningWorkflowRun,
    stage_identity: &str,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    kind: WorthQueryGraphProviderCallKind,
    scope_identity: &str,
) -> Result<
    (
        WorthQueryRunningWorkflowRun,
        WorthQueryBoundGraphExecutionReceipt,
    ),
    (
        String,
        worth_query_execution::facade::runtime::WorthQueryWorkflowRunTerminal,
    ),
> {
    let active = running
        .begin_stage_graph_execution(
            stage_identity,
            graph,
            WorthQueryManagedGraphCallRequest::new(kind, scope_identity),
        )
        .map_err(|failure| {
            let detail = failure.detail().to_owned();
            (detail, failure.into_running().abandon())
        })?;
    let mut outcome = active.advance();
    loop {
        outcome = match outcome {
            WorthQueryWorkflowGraphStepOutcome::Continue(paused) => paused.advance(),
            WorthQueryWorkflowGraphStepOutcome::ChunkReady(chunk) => chunk.acknowledge(),
            WorthQueryWorkflowGraphStepOutcome::Completed(completed) => {
                let receipt = completed.receipt().clone();
                return Ok((completed.into_running(), receipt));
            }
            WorthQueryWorkflowGraphStepOutcome::Cancelled(terminal)
            | WorthQueryWorkflowGraphStepOutcome::TimedOut(terminal)
            | WorthQueryWorkflowGraphStepOutcome::Exhausted(terminal)
            | WorthQueryWorkflowGraphStepOutcome::Degraded(terminal)
            | WorthQueryWorkflowGraphStepOutcome::Failed(terminal) => {
                let detail = format!(
                    "managed workflow graph execution terminated as {:?}",
                    terminal.kind()
                );
                return Err((detail, terminal));
            }
        };
    }
}
