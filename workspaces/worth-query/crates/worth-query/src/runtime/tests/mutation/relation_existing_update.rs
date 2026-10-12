use super::super::support::*;

fn task_relation_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntime {
    stateful_bridge_task_relation_runtime(request_execution)
}

mod batch_identity;
mod entity_updates;
mod verified_relation;
