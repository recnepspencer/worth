use super::super::support::*;

fn task_edge_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntime {
    stateful_bridge_task_edge_runtime(request_execution)
}

mod denial_trace;
mod entity_and_collection_denials;
mod symbol_declarations;
