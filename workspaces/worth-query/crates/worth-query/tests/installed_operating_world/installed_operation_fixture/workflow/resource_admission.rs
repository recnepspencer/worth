use worth_query::facade::{domain, runtime};

use super::{
    configured_runtime_without_executors, workflow_definition, workflow_package, GeometryDomain,
    ReadFamily, WorkflowRead, WorkflowStageExecutor,
};

pub(crate) fn workflow_workspace_with_parallel_provider<P>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    provider: P,
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
>
where
    P: domain::WorthQueryWorkflowParallelAdmissionProvider<
        GeometryDomain,
        WorkflowRead,
        ReadFamily,
    >,
{
    configured_runtime_without_executors(workflow_package(workflow_definition(None), true))
        .replayable_workflow_stage_executor(
            GeometryDomain,
            WorkflowRead,
            ReadFamily,
            WorkflowStageExecutor,
        )
        .workflow_parallel_admission_provider(GeometryDomain, WorkflowRead, ReadFamily, provider)
        .workspace(request_execution, name)
}
