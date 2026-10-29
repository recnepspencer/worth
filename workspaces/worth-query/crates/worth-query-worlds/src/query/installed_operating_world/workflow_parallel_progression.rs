pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::domain;

pub use super::installed_operation_fixture::{
    divergent_frontier_workspace, missing_parallel_provider_workspace,
    nondeterministic_workflow_workspace, serial_parallel_provider_workspace, workflow_workspace,
    GeometryDomain, ReadFamily, WorkflowRead,
};

pub fn complete_serial_trace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> String {
    let mut workspace = workflow_workspace(request_execution, name).unwrap();
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
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            &mut workspace,
        )
        .unwrap()
        .advance(
            "right",
            domain::WorthQueryWorkflowValue::Text("start".into()),
            &mut workspace,
        )
        .unwrap()
        .advance(
            "left",
            domain::WorthQueryWorkflowValue::Text("start".into()),
            &mut workspace,
        )
        .unwrap()
        .advance(
            "publish",
            domain::WorthQueryWorkflowValue::Text("join".into()),
            &mut workspace,
        )
        .unwrap()
        .complete()
        .unwrap()
        .semantic_identity()
        .to_string()
}
