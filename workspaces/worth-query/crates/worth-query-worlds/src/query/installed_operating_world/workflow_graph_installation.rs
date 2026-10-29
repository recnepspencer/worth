pub use super::installed_operation_fixture::{
    invalid_workflow_workspace, mismatched_workflow_determinism_workspace,
    mismatched_workflow_lowering_workspace, reversed_workflow_workspace, workflow_workspace,
    GeometryDomain, InvalidWorkflow, ReadFamily, WorkflowRead,
};

pub fn installed_graph(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    reversed: bool,
) -> Vec<String> {
    let mut workspace = if reversed {
        reversed_workflow_workspace(request_execution, name)
    } else {
        workflow_workspace(request_execution, name)
    }
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(
            crate::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .installed_graph()
        .stages()
        .iter()
        .map(|stage| stage.identity().to_string())
        .collect()
}
