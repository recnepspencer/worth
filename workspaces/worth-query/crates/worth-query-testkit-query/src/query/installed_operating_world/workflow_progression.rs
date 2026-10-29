pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::{domain, read};

pub use super::installed_operation_fixture::{
    workflow_workspace, GeometryDomain, ReadFamily, WorkflowRead,
};

pub fn complete_trace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    order: [&str; 2],
) -> String {
    let mut workspace = workflow_workspace(request_execution, name).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let mut run = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
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
        .unwrap();
    for stage in order {
        run = run
            .advance(
                stage,
                domain::WorthQueryWorkflowValue::Text("start".into()),
                &mut workspace,
            )
            .unwrap();
    }
    run.advance(
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
