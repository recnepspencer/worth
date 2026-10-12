use worth_query::facade::domain;

use super::executors::WorkflowStageExecutor;
use super::workflow::{valid_stages, workflow_package};
use super::workflow_parallel_providers::WorkflowParallelProvider;
use super::{configured_runtime_without_executors, GeometryDomain, ReadFamily, WorkflowRead};

pub fn failing_controlled_workflow_workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    failed_closes: usize,
) -> Result<
    worth_query::facade::consumer_kit::WorthQueryControlledTestWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    let workflow = domain::WorthQueryPortableWorkflowDefinition::new("start", valid_stages());
    configured_runtime_without_executors(workflow_package(workflow, true))
        .replayable_workflow_stage_executor(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowStageExecutor,
        )
        .workflow_parallel_admission_provider(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowParallelProvider,
        )
        .fail_next_live_closes(failed_closes)
        .controlled_workspace(request_execution, name)
}
