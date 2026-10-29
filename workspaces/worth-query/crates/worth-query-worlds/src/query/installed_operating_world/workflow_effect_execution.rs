pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::domain;

pub use super::installed_operation_fixture::{
    mutation_workflow_workspace, workflow_workspace, GeometryDomain, MutationFamily, ReadFamily,
    WorkflowMutation, WorkflowRead,
};

pub fn failing_stage_denial(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
    input: &str,
) -> domain::WorthQueryWorkflowAdvanceDenial {
    let mut workspace = workflow_workspace(request_execution, name).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let run = workspace
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
        .unwrap();
    match run.advance(
        "left",
        domain::WorthQueryWorkflowValue::Text(input.into()),
        &mut workspace,
    ) {
        TransitionOutcome::Failed(denial) => denial,
        TransitionOutcome::Denied(denial)
        | TransitionOutcome::Stale(denial)
        | TransitionOutcome::RebindRequired(denial) => {
            panic!("failing stage was refused before its expected executor failure: {denial:?}")
        }
        _ => panic!("failing stage did not produce an execution failure"),
    }
}
