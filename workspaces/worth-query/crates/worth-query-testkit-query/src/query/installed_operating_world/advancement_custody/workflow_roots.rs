//! Every workflow root is driven through its public host entry.
pub use super::super::workflow_projection_lifecycle::{promoted, settle_workflow};
pub use super::*;
pub type Run = domain::WorthQueryWorkflowRun<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

pub fn run_setup(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (worth_query::facade::runtime::WorthQueryWorkspace, Run) {
    let mut workspace = workflow_workspace(request_execution, "workflow-root-custody").unwrap();
    let run = bind(&workspace)
        .admit_workflow_resources(execution_resource_request(), &workspace)
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap();
    (workspace, run)
}
